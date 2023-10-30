use anyhow::{bail, Result};
use async_recursion::async_recursion;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use std::path::Path;
use tokio::fs;
use tracing::info;
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

    info!("Recreating dist/ directory");
    fs::remove_dir_all("dist").await.ok();
    fs::create_dir_all("dist").await?;

    info!("Copying static files");
    fs::create_dir_all("dist/static").await?;
    copy_dir("static", "dist/static").await?;

    fs::write("dist/index.html", templates::index().into_string()).await?;

    Ok(())
}

#[async_recursion]
async fn copy_dir<F, T>(from: F, to: T) -> Result<()>
where
    F: AsRef<Path> + Send + Sync,
    T: AsRef<Path> + Send,
{
    let mut dir = fs::read_dir(&from).await?;
    while let Some(item) = dir.next_entry().await? {
        let file_name = item.file_name();

        if file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let new_path = to.as_ref().join(file_name);
        if new_path.exists() {
            bail!("File or directory already exists: {:?}", new_path)
        }

        if item.path().is_dir() {
            fs::create_dir(&new_path).await?;
            copy_dir(item.path(), &new_path).await?;
        } else {
            let path = item.path();
            info!(path = item.path().to_str(), "Copying static file");
            fs::copy(path, new_path).await?;
        }
    }

    Ok(())
}
