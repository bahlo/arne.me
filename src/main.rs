use anyhow::{bail, Result};
use std::{fs, path::Path};

mod content;
mod layout;
mod templates;

use crate::content::Content;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = Content::parse(fs::read_dir("content")?)?;

    // Recreate dir
    fs::remove_dir_all("dist").ok();
    fs::create_dir_all("dist")?;

    // Copy static files
    copy_dir("static", "dist/")?;

    // Generate CSS
    let sass_options = grass::Options::default().load_path("styles/");
    let css = grass::from_path("styles/main.scss", &sass_options)?;
    fs::write("dist/main.css", css)?;

    // Generate HTML
    fs::write("dist/index.html", templates::index(&content).into_string())?;
    for article in &content.articles {
        fs::create_dir_all(format!("dist/articles/{}", article.slug))?;
        let path = format!("dist/articles/{}/index.html", article.slug);
        fs::write(&path, templates::article(article).into_string())?;
    }

    Ok(())
}

fn copy_dir<F, T>(from: F, to: T) -> Result<()>
where
    F: AsRef<Path> + Send + Sync,
    T: AsRef<Path> + Send,
{
    // TODO: Turn this into functional code
    let mut dir = fs::read_dir(&from)?;
    while let Some(item) = dir.next().transpose()? {
        let file_name = item.file_name();

        if file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let new_path = to.as_ref().join(file_name);
        if new_path.exists() {
            bail!("File or directory already exists: {:?}", new_path)
        }

        if item.path().is_dir() {
            fs::create_dir(&new_path)?;
            copy_dir(item.path(), &new_path)?;
        } else {
            let path = item.path();
            fs::copy(path, new_path)?;
        }
    }

    Ok(())
}
