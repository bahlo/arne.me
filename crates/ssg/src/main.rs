use anyhow::{bail, Result};
use clap::Parser;
use rayon::prelude::*;
use std::{cell::LazyCell, fs, path::Path, process::Command};
use templates::layout::Layout;
use timer::Timer;

mod blog;
mod library;
mod rss;
mod sitemap;
mod templates;
mod timer;
mod weekly;

use arneos::content::Content;

pub const GIT_SHA: LazyCell<String> = LazyCell::new(|| {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to eecute git command");
    String::from_utf8(output.stdout).expect("Failed to parse git output")
});
pub const GIT_SHA_SHORT: LazyCell<String> = LazyCell::new(|| GIT_SHA.chars().take(7).collect());

#[derive(Debug, Parser)]
struct Ssg {
    #[clap(long)]
    websocket_port: Option<u16>,
    #[clap(long, default_value = "false")]
    generate_missing_og_images: bool,
}

pub fn main() -> Result<()> {
    let ssg = Ssg::parse();

    // Download fonts if we have to
    // TODO: Instead of checking if a specific font exists, check that _any_
    //       dir exists.
    if !Path::new("static/fonts/rebond-grotesque").exists() {
        let mut timer = Timer::new("Downloading fonts");
        arneos::fonts::download_fonts()?;
        timer.end();
    }

    // Parse content
    let mut timer = Timer::new("Parsing content");
    let content = Content::parse(fs::read_dir("content")?)?;
    timer.end();

    // Recreate dir
    let mut timer = Timer::new("Recreating dist/");
    fs::remove_dir_all("dist").ok();
    fs::create_dir_all("dist")?;
    timer.end();

    // Copy static files
    let mut timer = Timer::new("Copying static files");
    copy_dir("static", "dist/")?;
    timer.end();

    // Generate CSS
    let mut timer = Timer::new("Generating CSS");
    let css_hash = pichu::render_sass("styles/main.scss", "dist/main.css")?;
    timer.end();

    let mut timer = Timer::new("Generating HTML");

    // Create layout
    let layout = Layout::new(css_hash, ssg.websocket_port, ssg.generate_missing_og_images);

    // Generate index
    fs::create_dir_all("dist")?;
    fs::write(
        "dist/index.html",
        layout
            .render(templates::index::render(&content)?)?
            .into_string(),
    )?;

    let _blog = pichu::glob("content/blog/*.md")?
        .parse_markdown::<blog::Blogpost>()?
        .sort_by_key_reverse(|post| post.frontmatter.published)
        .render_each(
            |post| blog::render_single(&layout, post),
            |post| format!("dist/blog/{}/index.html", post.basename),
        )?
        .render_all(
            |blog_posts| blog::render_all(&layout, blog_posts),
            "dist/blog/index.html",
        )?;

    let weekly = pichu::glob("content/weekly/*.md")?
        .parse_markdown::<weekly::Issue>()?
        .sort_by_key_reverse(|issue| issue.frontmatter.date)
        .render_each(
            |issue| weekly::render_single(&layout, issue),
            |issue| format!("dist/weekly/{}/index.html", issue.basename),
        )?
        .render_all(
            |issues| weekly::render_all(&layout, issues),
            "dist/weekly/index.html",
        )?;

    let _library = pichu::glob("content/library/*.md")?
        .parse_markdown::<library::Book>()?
        .render_each(
            |book| library::render_single(&layout, book),
            |book| format!("dist/library/{}/index.html", book.basename),
        )?
        .render_all(
            |books| library::render_all(&layout, books),
            format!("dist/library/index.html"),
        );

    // Generate pages
    content
        .pages
        .par_iter()
        .map(|page| {
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
            Ok(())
        })
        .collect::<Result<()>>()?;

    // Generate projects page
    fs::create_dir_all("dist/projects")?;
    fs::write(
        "dist/projects/index.html",
        layout
            .render(templates::project::render(&content.projects)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/projects")?;

    timer.end();

    // Generate RSS feeds
    let mut timer = Timer::new("Generating RSS feeds");
    fs::write("dist/blog/feed.xml", rss::render_blog(&content))?;
    pichu::write(rss::render_weekly(&weekly.items)?, "dist/weekly/feed.xml")?;
    fs::write("dist/library/feed.xml", rss::render_library(&content))?;
    timer.end();

    // Generate sitemap.xml
    let mut timer = Timer::new("Generating sitemap...");
    fs::write("dist/sitemap.xml", sitemap::render(&content)?)?;
    timer.end();

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
