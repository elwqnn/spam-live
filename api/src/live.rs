use std::{
    convert::Infallible,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use futures_util::{StreamExt, stream};
use sqlx::PgPool;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::{ApiError, AppState, ErrorBody};

pub const MAX_STREAMS: usize = 1_000;
pub const CAPACITY: usize = 256;

#[derive(Clone, Copy)]
enum Signal {
    Deck(i64),
    Votes { id: i64, upvotes: i64, downvotes: i64 },
}

pub struct Live {
    tx: Mutex<Option<broadcast::Sender<Signal>>>,
    open: AtomicUsize,
    max: usize,
    votes: tokio::sync::Mutex<()>,
}

impl Default for Live {
    fn default() -> Self {
        Self::new(MAX_STREAMS)
    }
}

struct Slot(Arc<Live>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.open.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Live {
    pub fn new(max: usize) -> Self {
        Self {
            tx: Mutex::new(Some(broadcast::channel(CAPACITY).0)),
            open: AtomicUsize::new(0),
            max,
            votes: tokio::sync::Mutex::new(()),
        }
    }

    fn sender(&self) -> MutexGuard<'_, Option<broadcast::Sender<Signal>>> {
        self.tx.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn send(&self, signal: Signal) {
        if let Some(tx) = self.sender().as_ref() {
            tx.send(signal).ok();
        }
    }

    pub fn notify(&self, deck_id: i64) {
        self.send(Signal::Deck(deck_id));
    }

    pub async fn votes(&self, db: &PgPool, id: i64) {
        let _order = self.votes.lock().await;
        let counts = sqlx::query_as(
            "SELECT count(*) FILTER (WHERE value = 1), count(*) FILTER (WHERE value = -1) FROM votes WHERE deck_id = $1",
        )
        .bind(id)
        .fetch_one(db)
        .await;
        if let Ok((upvotes, downvotes)) = counts {
            self.send(Signal::Votes { id, upvotes, downvotes });
        }
    }

    pub fn close(&self) {
        self.sender().take();
    }

    fn subscribe(self: &Arc<Self>) -> Option<(broadcast::Receiver<Signal>, Slot)> {
        self.open.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < self.max).then_some(n + 1)).ok()?;
        let slot = Slot(self.clone());
        let rx = self.sender().as_ref()?.subscribe();
        Some((rx, slot))
    }
}

#[utoipa::path(get, path = "/api/decks/events", tag = "decks",
    responses((status = 200, content_type = "text/event-stream", body = String,
            description = "Server-Sent Events. The first frame sets retry to 5000 ms. An event named decks has the ID \
                of a deck that came into the public decks, left them, or changed while public, \
                or 0 after missed events or an account deletion. Fetch the list again on each decks event. \
                An event named votes comes after each vote change on a public deck. Its data is JSON \
                {\"id\":1,\"upvotes\":3,\"downvotes\":1}: the deck ID and its new vote counts. Not cached."),
        (status = 503, body = ErrorBody, description = "Too many open streams")))]
pub async fn deck_events(State(s): State<AppState>) -> Result<Response, ApiError> {
    let (rx, slot) =
        s.live.subscribe().ok_or_else(|| ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "too many open streams"))?;
    let start = stream::once(async { Ok(Event::default().retry(Duration::from_secs(5))) });
    let signals = stream::unfold((rx, slot), |(mut rx, slot)| async move {
        let event = match rx.recv().await {
            Ok(Signal::Deck(id)) => Event::default().event("decks").data(id.to_string()),
            Ok(Signal::Votes { id, upvotes, downvotes }) => Event::default()
                .event("votes")
                .data(format!(r#"{{"id":{id},"upvotes":{upvotes},"downvotes":{downvotes}}}"#)),
            Err(RecvError::Lagged(_)) => Event::default().event("decks").data("0"),
            Err(RecvError::Closed) => return None,
        };
        Some((Ok::<_, Infallible>(event), (rx, slot)))
    });
    let events = start.chain(signals);
    Ok(([(header::CACHE_CONTROL, "no-store")], Sse::new(events).keep_alive(KeepAlive::default())).into_response())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/decks/events", get(deck_events))
}
