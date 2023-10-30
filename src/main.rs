use axum::{routing::get, Router, Server};
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

mod content;
mod layout;
mod routes;

use crate::content::Content;

static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/content");

lazy_static! {
    pub static ref CONTENT: Content = Content::parse(&PROJECT_DIR).expect("Failed to load content");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let articles = &CONTENT.articles;
    dbg!(articles);

    let app = Router::new().route("/", get(routes::index));

    info!(port = 3000, "Starting server");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
