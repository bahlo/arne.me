use anyhow::{bail, Result};
use std::{fs, path::Path};

mod content;
mod layout;
mod rss;
mod templates;

use crate::content::Content;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse content
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

    // Generate articles
    fs::write("dist/index.html", templates::index(&content)?.into_string())?;
    for article in &content.articles {
        fs::create_dir_all(format!("dist/articles/{}", article.slug))?;
        let path = format!("dist/articles/{}/index.html", article.slug);
        fs::write(&path, templates::article(article)?.into_string())?;
    }

    // Generate weekly
    fs::create_dir_all("dist/weekly")?;
    fs::write(
        "dist/weekly/index.html",
        templates::weekly_index(&content)?.into_string(),
    )?;
    for weekly_issue in &content.weekly {
        fs::create_dir_all(format!("dist/weekly/{}", weekly_issue.num))?;
        let path = format!("dist/weekly/{}/index.html", weekly_issue.num);
        fs::write(&path, templates::weekly(weekly_issue)?.into_string())?;
    }

    // Generate pages
    for page in &content.pages {
        fs::create_dir_all(format!("dist/{}", page.slug))?;
        let path = format!("dist/{}/index.html", page.slug);
        fs::write(&path, templates::page(page)?.into_string())?;
    }

    // Generate projects page
    fs::create_dir_all("dist/projects")?;
    fs::write(
        "dist/projects/index.html",
        templates::projects(&content.projects)?.into_string(),
    )?;

    // Generate RSS feeds
    fs::write("dist/feed.xml", rss::render_feed(&content))?;

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
