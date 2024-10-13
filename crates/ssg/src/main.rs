use anyhow::{bail, Result};
use std::{cell::LazyCell, env, fs, path::Path, process::Command};
use templates::layout::Layout;

mod rss;
mod sitemap;
mod templates;

use arneos::content::Content;

pub const GIT_SHA: LazyCell<String> = LazyCell::new(|| {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to eecute git command");
    String::from_utf8(output.stdout).expect("Failed to parse git output")
});
pub const GIT_SHA_SHORT: LazyCell<String> = LazyCell::new(|| GIT_SHA.chars().take(7).collect());

pub fn main() -> Result<()> {
    // Do we have a websocket port?
    let args: Vec<String> = env::args().collect();
    let websocket_port = match (args.get(1).map(|s| s.as_str()), args.get(2)) {
        (Some("--websocket-port"), Some(port)) => Some(port.parse::<u16>()?),
        _ => None,
    };

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
    let css_hash: String = blake3::hash(css.as_bytes())
        .to_string()
        .chars()
        .take(16)
        .collect();
    fs::write("dist/main.css", css)?;

    // Create layout
    let layout = Layout::new(css_hash, websocket_port);

    // Generate index
    fs::create_dir_all("dist")?;
    fs::write(
        "dist/index.html",
        layout
            .render(templates::index::render(&content)?)?
            .into_string(),
    )?;

    // Generate blog
    fs::create_dir_all("dist/blog")?;
    fs::write(
        "dist/blog/index.html",
        layout
            .render(templates::blog::render_page(&content)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/blog")?;
    for blogpost in &content.blog {
        fs::create_dir_all(format!("dist/blog/{}", blogpost.slug))?;
        let path = format!("dist/blog/{}/index.html", blogpost.slug);
        fs::write(
            &path,
            layout
                .render(templates::blog::render(blogpost)?)?
                .into_string(),
        )?;
        fs::create_dir_all(format!("static/blog/{}", blogpost.slug))?;
    }

    // Generate weekly
    fs::create_dir_all("dist/weekly")?;
    fs::write(
        "dist/weekly/index.html",
        layout
            .render(templates::weekly::render_index(&content)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/weekly")?;
    for weekly_issue in &content.weekly {
        fs::create_dir_all(format!("dist/weekly/{}", weekly_issue.num))?;
        let path = format!("dist/weekly/{}/index.html", weekly_issue.num);
        fs::write(
            &path,
            layout
                .render(templates::weekly::render(weekly_issue)?)?
                .into_string(),
        )?;
        fs::create_dir_all(format!("static/weekly/{}", weekly_issue.num))?;
    }

    // Generate book reviews
    fs::create_dir_all("dist/library")?;
    fs::write(
        "dist/library/index.html",
        layout
            .render(templates::library::render_index(&content)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/library")?;
    for book in &content.library {
        fs::create_dir_all(format!("dist/library/{}", book.slug))?;
        let path = format!("dist/library/{}/index.html", book.slug);
        fs::write(
            &path,
            layout
                .render(templates::library::render(book)?)?
                .into_string(),
        )?;
        fs::create_dir_all(format!("static/library/{}", book.slug))?;
    }

    // Generate home screens
    fs::create_dir_all("dist/home-screens")?;
    fs::write(
        "dist/home-screens/index.html",
        layout
            .render(templates::home_screen::render_index(&content)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/home-screens")?;
    for home_screen in &content.home_screens {
        fs::create_dir_all(format!("dist/home-screens/{}", home_screen.slug))?;
        let path = format!("dist/home-screens/{}/index.html", home_screen.slug);
        fs::write(
            &path,
            layout
                .render(templates::home_screen::render(home_screen)?)?
                .into_string(),
        )?;
        fs::create_dir_all(format!("static/home-screens/{}", home_screen.slug))?;
    }

    // Generate pages
    for page in &content.pages {
        let path = match page.slug.as_str() {
            "404" => "dist/404.html".to_string(),
            _ => {
                fs::create_dir_all(format!("dist/{}", page.slug))?;
                format!("dist/{}/index.html", page.slug)
            }
        };

        fs::write(
            &path,
            layout.render(templates::page::render(page)?)?.into_string(),
        )?;
        fs::create_dir_all(format!("static/{}", page.slug))?;
    }

    // Generate projects page
    fs::create_dir_all("dist/projects")?;
    fs::write(
        "dist/projects/index.html",
        layout
            .render(templates::project::render(&content.projects)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/projects")?;

    // Generate RSS feeds
    fs::write("dist/feed.xml", rss::render_blog(&content))?;
    fs::write("dist/weekly/feed.xml", rss::render_weekly(&content)?)?;
    fs::write("dist/library/feed.xml", rss::render_library(&content))?;

    // Generate sitemap.xml
    fs::write("dist/sitemap.xml", sitemap::render(&content)?)?;

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

        let file_name_str = file_name.to_string_lossy();
        if file_name_str.starts_with('.') && file_name_str != ".well-known" {
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
