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
use tokio::sync::broadcast::{self, error::RecvError};

use crate::{ApiError, AppState, ErrorBody};

pub const MAX_STREAMS: usize = 1_000;
const CAPACITY: usize = 16;

pub struct Live {
    tx: Mutex<Option<broadcast::Sender<i64>>>,
    open: AtomicUsize,
    max: usize,
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
        Self { tx: Mutex::new(Some(broadcast::channel(CAPACITY).0)), open: AtomicUsize::new(0), max }
    }

    fn sender(&self) -> MutexGuard<'_, Option<broadcast::Sender<i64>>> {
        self.tx.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn notify(&self, deck_id: i64) {
        if let Some(tx) = self.sender().as_ref() {
            tx.send(deck_id).ok();
        }
    }

    pub fn close(&self) {
        self.sender().take();
    }

    fn subscribe(self: &Arc<Self>) -> Option<(broadcast::Receiver<i64>, Slot)> {
        self.open.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < self.max).then_some(n + 1)).ok()?;
        let slot = Slot(self.clone());
        let rx = self.sender().as_ref()?.subscribe();
        Some((rx, slot))
    }
}

#[utoipa::path(get, path = "/api/decks/events", tag = "decks",
    responses((status = 200, content_type = "text/event-stream", body = String,
            description = "Server-Sent Events. The first frame sets retry to 5000 ms. Each event has the name decks. \
                Its data is the ID of a deck that came into the public decks, left them, or changed while public, \
                or 0 after missed events or an account deletion. Fetch the list again on each event. Not cached."),
        (status = 503, body = ErrorBody, description = "Too many open streams")))]
pub async fn deck_events(State(s): State<AppState>) -> Result<Response, ApiError> {
    let (rx, slot) =
        s.live.subscribe().ok_or_else(|| ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "too many open streams"))?;
    let start = stream::once(async { Ok(Event::default().retry(Duration::from_secs(5))) });
    let signals = stream::unfold((rx, slot), |(mut rx, slot)| async move {
        let id = match rx.recv().await {
            Ok(id) => id,
            Err(RecvError::Lagged(_)) => 0,
            Err(RecvError::Closed) => return None,
        };
        Some((Ok::<_, Infallible>(Event::default().event("decks").data(id.to_string())), (rx, slot)))
    });
    let events = start.chain(signals);
    Ok(([(header::CACHE_CONTROL, "no-store")], Sse::new(events).keep_alive(KeepAlive::default())).into_response())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/decks/events", get(deck_events))
}
