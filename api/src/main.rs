use std::{error::Error, path::PathBuf, time::Duration};

use api::{AppState, Config, catalog};
use sqlx::postgres::PgPoolOptions;
use tokio::signal::unix::{SignalKind, signal};
use utoipa::OpenApi;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn required(key: &str) -> Result<String, String> {
    env(key).ok_or_else(|| format!("{key} is not set"))
}

#[cfg(feature = "seed")]
async fn seed() -> Result<(), Box<dyn Error>> {
    let options = api::seed::check_host(&required("DATABASE_URL")?)?;
    let catalog = catalog::Catalog::load(&PathBuf::from(env("CATALOG_DIR").unwrap_or_else(|| "catalog".into())))?;
    let db = PgPoolOptions::new().max_connections(2).connect_with(options).await?;
    sqlx::migrate!().run(&db).await?;
    api::seed::run(&db, &catalog).await?;
    for (table, n) in api::seed::counts(&db).await? {
        println!("{table}: {n}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().any(|a| a == "--openapi") {
        println!("{}", api::ApiDoc::openapi().to_pretty_json()?);
        return Ok(());
    }
    #[cfg(feature = "seed")]
    if std::env::args().any(|a| a == "--seed") {
        return seed().await;
    }
    tracing_subscriber::fmt().init();

    let database_url = required("DATABASE_URL")?;
    let public_url = required("PUBLIC_URL")?.trim_end_matches('/').to_string();
    reqwest::Url::parse(&public_url).map_err(|e| format!("PUBLIC_URL is invalid: {e}"))?;
    let steam_openid_url = env("STEAM_OPENID_URL").unwrap_or_else(|| "https://steamcommunity.com/openid/login".into());
    reqwest::Url::parse(&steam_openid_url).map_err(|e| format!("STEAM_OPENID_URL is invalid: {e}"))?;
    let github_release_url = env("GITHUB_RELEASE_URL")
        .unwrap_or_else(|| "https://api.github.com/repos/elwqnn/spam-live/releases/latest".into());
    reqwest::Url::parse(&github_release_url).map_err(|e| format!("GITHUB_RELEASE_URL is invalid: {e}"))?;
    let github_api_url =
        env("GITHUB_API_URL").unwrap_or_else(|| "https://api.github.com".into()).trim_end_matches('/').to_string();
    reqwest::Url::parse(&github_api_url).map_err(|e| format!("GITHUB_API_URL is invalid: {e}"))?;
    let github_feedback_repo = env("GITHUB_FEEDBACK_REPO").unwrap_or_else(|| "elwqnn/spam-live".into());
    let repo_part = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b));
    if !github_feedback_repo.split_once('/').is_some_and(|(owner, name)| repo_part(owner) && repo_part(name)) {
        return Err("GITHUB_FEEDBACK_REPO must be owner/name".into());
    }
    let desktop_min_version = env("DESKTOP_MIN_VERSION").unwrap_or_else(|| "0.0.0".into());
    catalog::parse_version(&desktop_min_version).ok_or("DESKTOP_MIN_VERSION is invalid")?;
    let catalog_dir = PathBuf::from(env("CATALOG_DIR").unwrap_or_else(|| "catalog".into()));
    let cors_origins: Vec<String> =
        env("CORS_ORIGINS").map(|v| v.split(',').map(|o| o.trim().to_string()).collect()).unwrap_or_default();
    if let Some(bad) = cors_origins
        .iter()
        .find(|o| reqwest::Url::parse(o).map_or(true, |url| url.origin().ascii_serialization() != **o))
    {
        return Err(format!("CORS_ORIGINS has an invalid origin: {bad}").into());
    }
    let bind = env("BIND").unwrap_or_else(|| "0.0.0.0:8080".into());

    let catalog = catalog::Catalog::load(&catalog_dir)?;
    let db = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!().run(&db).await?;

    let config = Config {
        public_url,
        steam_api_key: env("STEAM_API_KEY"),
        steam_openid_url,
        github_release_url,
        github_api_url,
        github_feedback_repo,
        github_feedback_token: env("GITHUB_FEEDBACK_TOKEN"),
        desktop_min_version,
        max_pending_device_codes: 1000,
        cors_origins,
    };
    let state = AppState::new(db, config, catalog);

    let replays_db = state.db.clone();
    tokio::spawn(async move {
        match api::matches::clean_stored_replays(&replays_db).await {
            Ok((cleaned, deleted)) => tracing::info!(cleaned, deleted, "stored replays are clean"),
            Err(e) => tracing::error!(error = %e, "stored replay cleanup failed"),
        }
    });

    let cleanup_db = state.db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        loop {
            interval.tick().await;
            if let Err(e) = api::cleanup(&cleanup_db).await {
                tracing::error!(error = %e, "cleanup failed");
            }
        }
    });

    let feedback_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            api::feedback::retry_pending(&feedback_state).await;
        }
    });

    let analytics_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut hourly = api::analytics::Hourly::new(chrono::Utc::now());
        loop {
            interval.tick().await;
            if let Err(e) = api::analytics::tick(&analytics_state, &mut hourly, chrono::Utc::now()).await {
                tracing::error!(error = %e, "analytics tick failed");
            }
        }
    });

    let live = state.live.clone();
    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("listening on {bind}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
            live.close();
        })
        .await?;
    Ok(())
}
