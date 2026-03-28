mod config;
mod handlers;
mod middleware;
mod routes;
mod extractors;

// use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use vpn_data::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Starting VPN API server");
    tracing::info!("Database URL: {}", config.database_url);

    // Connect to database
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connected successfully");

    // Build router
    let router = routes::build_router(db).layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
