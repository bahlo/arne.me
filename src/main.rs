#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
use anyhow::Result;
use clap::Parser;
use comrak::markdown_to_html;
use layout::Layout;
use maud::Markup;
use pichu::{Markdown, MarkdownError};
use std::{fs, path::Path, process::Command, sync::LazyLock};
use timer::Timer;

use crate::content::{blog, index, library, page, project, weekly};

mod automate;
mod content;
mod fonts;
mod layout;
mod og;
mod rss;
mod sitemap;
mod timer;
mod watch;

pub static GIT_SHA: LazyLock<String> = LazyLock::new(|| {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to eecute git command");
    String::from_utf8(output.stdout).expect("Failed to parse git output")
});
pub static GIT_SHA_SHORT: LazyLock<String> = LazyLock::new(|| GIT_SHA.chars().take(7).collect());

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    #[clap(name = "build")]
    Build {
        #[clap(long)]
        websocket_port: Option<u16>,
        #[clap(long, default_value = "false")]
        generate_missing_og_images: bool,
    },
    #[clap(name = "watch")]
    Watch,
    #[clap(name = "automate")]
    Automate {
        #[clap(long, short, group = "subject")]
        before_sha: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            websocket_port,
            generate_missing_og_images,
        } => build(websocket_port, generate_missing_og_images),
        Commands::Watch => watch::watch(),
        Commands::Automate { before_sha } => automate::automate_before_sha(&before_sha),
    }
}

fn build(websocket_port: Option<u16>, generate_missing_og_images: bool) -> Result<()> {
    // Download fonts if we have to
    // TODO: Instead of checking if a specific font exists, check that _any_
    //       dir exists.
    if !Path::new("static/fonts/rebond-grotesque").exists() {
        let mut timer = Timer::new("Downloading fonts");
        fonts::download_fonts()?;
        timer.end();
    }

    // Recreate dir
    let mut timer = Timer::new("Recreating dist/");
    fs::remove_dir_all("dist").ok();
    fs::create_dir_all("dist")?;
    timer.end();

    // Copy static files
    let mut timer = Timer::new("Copying static files");
    pichu::copy_dir("static", "dist/")?;
    timer.end();

    // Generate CSS
    let mut timer = Timer::new("Generating CSS");
    let css_hash = pichu::render_sass("styles/main.scss", "dist/main.css")?;
    timer.end();

    let mut timer = Timer::new("Generating HTML");

    // Create layout
    let layout = Layout::new(css_hash, websocket_port, generate_missing_og_images);

    let blog = pichu::glob("content/blog/*.md")?
        .parse_markdown::<blog::Blogpost>()?
        .sort_by_key_reverse(|post| post.frontmatter.published)
        .try_render_each(
            |post| -> Result<Markup> {
                let html = blog::render_single(&layout, post)?;
                Ok(html)
            },
            |post| format!("dist/blog/{}/index.html", post.basename),
        )?
        .try_render_all(
            |blog_posts| -> Result<Markup> {
                let html = blog::render_all(&layout, blog_posts)?;
                Ok(html)
            },
            "dist/blog/index.html",
        )?
        .into_vec();

    let weekly = pichu::glob("content/weekly/*.md")?
        .try_parse::<Markdown<weekly::Issue>, MarkdownError>(|path| {
            let mut markdown = pichu::parse_markdown::<weekly::Issue>(path)?;
            markdown.frontmatter.categories = markdown
                .frontmatter
                .categories
                .into_iter()
                .map(|mut category| {
                    category.stories = category
                        .stories
                        .into_iter()
                        .map(|mut story| {
                            story.description_html =
                                markdown_to_html(&story.description, &comrak::Options::default());
                            story
                        })
                        .collect();
                    category
                })
                .collect();
            Ok(markdown)
        })?
        .sort_by_key_reverse(|issue| issue.frontmatter.date)
        .try_render_each(
            |issue| -> Result<Markup> {
                let html = weekly::render_single(&layout, issue)?;
                Ok(html)
            },
            |issue| format!("dist/weekly/{}/index.html", issue.basename),
        )?
        .try_render_all(
            |issues| -> Result<Markup> {
                let html = weekly::render_all(&layout, issues)?;
                Ok(html)
            },
            "dist/weekly/index.html",
        )?
        .into_vec();

    let library = pichu::glob("content/library/*.md")?
        .parse_markdown::<library::Book>()?
        .sort_by_key_reverse(|issue| issue.frontmatter.read)
        .try_render_each(
            |book| -> Result<Markup> {
                let html = library::render_single(&layout, book)?;
                Ok(html)
            },
            |book| format!("dist/library/{}/index.html", book.basename),
        )?
        .try_render_all(
            |books| -> Result<Markup> {
                let html = library::render_all(&layout, books)?;
                Ok(html)
            },
            "dist/library/index.html",
        )?
        .into_vec();

    let pages = pichu::glob("content/*.md")?
        .parse_markdown::<page::Page>()?
        .try_render_each(
            |page| -> Result<Markup> {
                let html = page::render_each(&layout, page)?;
                Ok(html)
            },
            |page| match page.basename.as_str() {
                "404" => "dist/404.html".to_string(),
                _ => format!("dist/{}/index.html", page.basename),
            },
        )?
        .into_vec();

    let _projects = pichu::glob("content/projects/*.md")?
        .parse_markdown::<project::Project>()?
        .try_render_all(
            |projects| -> Result<Markup> {
                let html = project::render_all(&layout, projects)?;
                Ok(html)
            },
            "dist/projects/index.html",
        )?
        .into_vec();

    pichu::write(
        "dist/index.html",
        index::render(&layout, &blog, &weekly, &library)?.into_string(),
    )?;

    timer.end();

    // Generate RSS feeds
    let mut timer = Timer::new("Generating RSS feeds");
    pichu::write("dist/blog/feed.xml", rss::render_blog(&blog))?;
    pichu::write("dist/weekly/feed.xml", rss::render_weekly(&weekly)?)?;
    pichu::write("dist/library/feed.xml", rss::render_library(&library))?;
    timer.end();

    // Generate sitemap.xml
    let mut timer = Timer::new("Generating sitemap...");
    pichu::write(
        "dist/sitemap.xml",
        sitemap::render(&blog, &weekly, &library, &pages)?,
    )?;
    timer.end();

    Ok(())
}
