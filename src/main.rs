//! application-a: greets by name, plainly, and counts every greeting in
//! postgres.

use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Deserialize)]
struct HelloQuery {
    name: Option<String>,
}

/// What `GET /api/a/hello` answers; the SDK's `Hello` mirrors it.
#[derive(Serialize)]
struct Hello {
    message: String,
    count: i64,
}

fn message(name: &str) -> String {
    hello::greet(name, hello::Style::Plain)
}

async fn hello(
    State(db): State<PgPool>,
    Query(query): Query<HelloQuery>,
) -> Result<Json<Hello>, StatusCode> {
    let name = query.name.unwrap_or_else(|| "world".to_string());
    let count = record(&db, &name).await.map_err(|e| {
        tracing::error!(error = %e, "cannot record the greeting");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(Hello {
        message: message(&name),
        count,
    }))
}

/// Record a greeting of `name`; the number of greetings so far.
async fn record(db: &PgPool, name: &str) -> sqlx::Result<i64> {
    sqlx::query("INSERT INTO greetings (name) VALUES ($1)")
        .bind(name)
        .execute(db)
        .await?;
    sqlx::query_scalar("SELECT count(*) FROM greetings")
        .fetch_one(db)
        .await
}

async fn ready(State(db): State<PgPool>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// The database at `url`, waiting for it to accept connections: it may
/// start at the same time as this service.
async fn connect(url: &str) -> sqlx::Result<PgPool> {
    let mut attempt = 0;
    loop {
        match PgPoolOptions::new().max_connections(5).connect(url).await {
            Ok(db) => return Ok(db),
            Err(e) if attempt < 30 => {
                attempt += 1;
                tracing::warn!(error = %e, attempt, "database not ready");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn shutdown() {
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("a SIGTERM handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init("application-a", logging::Format::Pretty);
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is not set")?;
    let db = connect(&url).await?;
    sqlx::migrate!().run(&db).await?;

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let app = Router::new()
        .route("/api/a/hello", get(hello))
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(ready))
        .with_state(db);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_plainly() {
        assert_eq!(message("Ada"), "Hello, Ada!");
    }
}
