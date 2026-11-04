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
    let desktop_min_version = env("DESKTOP_MIN_VERSION").unwrap_or_else(|| "0.0.0".into());
    catalog::parse_version(&desktop_min_version).ok_or("DESKTOP_MIN_VERSION is invalid")?;
    let catalog_dir = PathBuf::from(env("CATALOG_DIR").unwrap_or_else(|| "catalog".into()));
    let bind = env("BIND").unwrap_or_else(|| "0.0.0.0:8080".into());

    let catalog = catalog::Catalog::load(&catalog_dir)?;
    let db = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!().run(&db).await?;

    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        loop {
            interval.tick().await;
            if let Err(e) = api::cleanup(&cleanup_db).await {
                tracing::error!(error = %e, "cleanup failed");
            }
        }
    });

    let config = Config {
        public_url,
        steam_api_key: env("STEAM_API_KEY"),
        steam_openid_url,
        github_release_url,
        desktop_min_version,
        max_pending_device_codes: 1000,
    };
    let app = api::router(AppState::new(db, config, catalog));
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("listening on {bind}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
        })
        .await?;
    Ok(())
}
