use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use tokio::fs;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

mod content;
mod layout;
mod templates;

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

    fs::remove_dir_all("dist").await.ok();
    fs::create_dir_all("dist").await?;
    fs::write("dist/index.html", templates::index().into_string()).await?;

    Ok(())
}
