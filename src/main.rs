use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::Parser;
use std::{
    cell::LazyCell,
    env,
    fs::{self, File},
    io,
    path::Path,
    process::Command,
};
use templates::layout::Layout;
use zip::ZipArchive;

mod content;
#[cfg(feature = "export-weekly-feeds")]
mod export_weekly_feeds;
mod og;
mod rss;
mod sitemap;
mod templates;
#[cfg(feature = "watch")]
mod watch;
#[cfg(feature = "send-webmentions")]
mod webmentions;

use content::Content;
#[cfg(feature = "export-weekly-feeds")]
use export_weekly_feeds::export_weekly_feeds;
#[cfg(feature = "send-webmentions")]
use webmentions::send_webmentions;

pub const GIT_SHA: LazyCell<String> = LazyCell::new(|| {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to eecute git command");
    String::from_utf8(output.stdout).expect("Failed to parse git output")
});
pub const GIT_SHA_SHORT: LazyCell<String> = LazyCell::new(|| GIT_SHA.chars().take(7).collect());

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum NewCommand {
    #[clap(name = "weekly")]
    Weekly,
    #[clap(name = "og-image")]
    OgImage { path: String },
}

#[derive(Debug, Parser)]
enum ExportCommand {
    #[clap(name = "weekly")]
    Weekly { num: Option<u16> },
    #[cfg(feature = "export-weekly-feeds")]
    #[clap(name = "weekly-feeds")]
    WeeklyFeeds { num: Option<u16> },
}

#[derive(Debug, Parser)]
enum Commands {
    #[clap(name = "build")]
    Build {
        #[clap(long)]
        websocket_port: Option<u16>,
    },
    #[cfg(feature = "watch")]
    #[clap(name = "watch")]
    Watch,
    #[clap(name = "download-fonts")]
    DownloadFonts,
    #[cfg(feature = "send-webmentions")]
    #[clap(name = "send-webmentions")]
    SendWebmentions {
        path: String,
        #[clap(long, short, default_value = "false")]
        dry_run: bool,
    },
    #[command(subcommand)]
    #[clap(name = "new")]
    New(NewCommand),
    #[command(subcommand)]
    #[clap(name = "export")]
    Export(ExportCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { websocket_port } => build(websocket_port),
        #[cfg(feature = "watch")]
        Commands::Watch => watch::watch(),
        Commands::DownloadFonts => download_fonts(),
        #[cfg(feature = "send-webmentions")]
        Commands::SendWebmentions { path, dry_run } => send_webmentions(path, dry_run),
        Commands::New(new) => match new {
            NewCommand::Weekly => new_weekly(),
            NewCommand::OgImage { path } => new_og_image(path),
        },
        Commands::Export(export) => match export {
            ExportCommand::Weekly { num } => export_weekly(num),
            #[cfg(feature = "export-weekly-feeds")]
            ExportCommand::WeeklyFeeds { num } => export_weekly_feeds(num),
        },
    }
}

pub fn build(websocket_port: Option<u16>) -> Result<()> {
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
    fs::create_dir_all("dist/book-reviews")?;
    fs::write(
        "dist/book-reviews/index.html",
        layout
            .render(templates::book_review::render_index(&content)?)?
            .into_string(),
    )?;
    fs::create_dir_all("static/book-reviews")?;
    for book_review in &content.book_reviews {
        fs::create_dir_all(format!("dist/book-reviews/{}", book_review.slug))?;
        let path = format!("dist/book-reviews/{}/index.html", book_review.slug);
        fs::write(
            &path,
            layout
                .render(templates::book_review::render(book_review)?)?
                .into_string(),
        )?;
        fs::create_dir_all(format!("static/book-reviews/{}", book_review.slug))?;
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
    fs::write(
        "dist/book-reviews/feed.xml",
        rss::render_book_reviews(&content),
    )?;

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

fn export_weekly(num: Option<u16>) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;

    // Default to the latest weekly issue
    let latest_issue = content
        .weekly
        .first()
        .ok_or(anyhow!("No weekly issues found"))?
        .num;
    let num = num.unwrap_or(latest_issue);

    let weekly_issue = content
        .weekly
        .iter()
        .find(|issue| issue.num == num)
        .ok_or_else(|| anyhow!("Weekly issue not found"))?;

    println!("{}", weekly_issue.content);
    println!();

    if let Some(quote_of_the_week) = &weekly_issue.quote_of_the_week {
        println!("## Quote of the Week");
        println!();
        quote_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!("> — {}", quote_of_the_week.author);
    } else if let Some(toot_of_the_week) = &weekly_issue.toot_of_the_week {
        println!("## Toot of the Week");
        println!();
        toot_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!(
            "> — [{}]({})",
            toot_of_the_week.author, toot_of_the_week.url
        );
    } else if let Some(tweet_of_the_week) = &weekly_issue.tweet_of_the_week {
        println!("## Tweet of the Week");
        println!();
        tweet_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!(
            "> — [{}]({})",
            tweet_of_the_week.author, tweet_of_the_week.url
        );
    }
    println!();
    weekly_issue.categories.iter().for_each(|category| {
        println!("## {}", category.title);
        category.stories.iter().for_each(|story| {
            println!("### [{}]({})", story.title, story.url);
            println!(
                "{} min · {}",
                story.reading_time_minutes,
                story.url.host().unwrap()
            );
            println!();
            println!("{}", story.description);
        });
        println!();
    });

    Ok(())
}

fn download_fonts() -> Result<()> {
    let zip_url = env::var("FONT_ZIP_URL")?;
    let destination = Path::new("./static/fonts");

    let response = ureq::get(&zip_url).call()?;
    let mut reader = response.into_reader();

    let zip_path = Path::join(&env::temp_dir(), "arne-me-fonts.zip");
    let mut temp_file = File::create(&zip_path)?;
    io::copy(&mut reader, &mut temp_file)?;

    let zip_file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    fs::remove_file(zip_path)?;
    Ok(())
}

pub fn new_weekly() -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;
    let num = content
        .weekly
        .first()
        .map(|issue| issue.num)
        .ok_or(anyhow!("No weekly issues found"))?
        + 1;

    let path = Path::new("content")
        .join("weekly")
        .join(format!("{}.md", num));
    if path.exists() {
        bail!("Weekly issue already exists at: {:?}", path);
    }

    let date = Utc::now().format("%Y-%m-%d").to_string();
    let content = format!(
        r#"---
title: "{num} /"
date: "{date}"
# tootOfTheWeek:
#   text:
#   url:
#   author:
categories:
  - title: "Cutting Room Floor"
    stories: []
---
    "#
    );

    fs::write(&path, content)?;
    println!("Created new weekly issue at: {:?}", path);
    Ok(())
}

fn new_og_image(path: impl AsRef<str>) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;

    let (kind, slug) = path
        .as_ref()
        .split_once("/")
        .unwrap_or_else(|| ("", path.as_ref()));

    // TODO: Support `*` to generate everything including index pages
    // (/, /blog, /weekly, /book-reviews)

    match kind {
        "weekly" => {
            let weekly_issue = content
                .weekly
                .iter()
                .find(|issue| issue.num.to_string() == slug)
                .ok_or(anyhow!("Can't find weekly issue"))?;
            og::generate(
                &weekly_issue.title,
                format!("static/weekly/{}/og-image.png", weekly_issue.num),
            )?;
        }
        "blog" => {
            let blogpost = content
                .blog
                .iter()
                .find(|blogpost| blogpost.slug == slug)
                .ok_or(anyhow!("Can't find blogpost"))?;
            og::generate(
                &blogpost.title,
                format!("static/blog/{}/og-image.png", blogpost.slug),
            )?;
        }
        "book-reviews" => {
            let book_review = content
                .book_reviews
                .iter()
                .find(|book_review| book_review.slug == slug)
                .ok_or(anyhow!("Can't find book review"))?;
            og::generate(
                &book_review.title,
                format!("static/book-reviews/{}/og-image.png", book_review.slug),
            )?;
        }
        "" => {
            let page = content
                .pages
                .iter()
                .find(|page| page.slug == slug)
                .ok_or(anyhow!("Can't find page"))?;
            og::generate(&page.title, format!("static/{}/og-image.png", page.slug))?;
        }
        _ => bail!("No idea what to do here"),
    }

    Ok(())
}
