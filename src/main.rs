use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::Parser;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
    process::Command,
};
use tempdir::TempDir;
use templates::layout::Layout;
use zip::ZipArchive;

mod content;
mod export_weekly_opml;
mod rss;
mod sitemap;
mod templates;
#[cfg(feature = "watch")]
mod watch;
#[cfg(feature = "send-webmentions")]
mod webmentions;

use crate::content::Content;
use export_weekly_opml::export_weekly_opml;
#[cfg(feature = "send-webmentions")]
use webmentions::send_webmentions;

lazy_static! {
    pub static ref GIT_SHA: String = {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .expect("Failed to eecute git command");
        String::from_utf8(output.stdout).expect("Failed to parse git output")
    };
    pub static ref GIT_SHA_SHORT: String = GIT_SHA.chars().take(7).collect();
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum NewCommand {
    #[clap(name = "weekly")]
    Weekly,
    #[clap(name = "home-screen")]
    HomeScreen,
}

#[derive(Debug, Parser)]
enum ExportCommand {
    #[clap(name = "weekly")]
    Weekly { num: Option<u16> },
    #[clap(name = "weekly-opml")]
    WeeklyOpml,
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
            NewCommand::HomeScreen => new_home_screen(),
        },
        Commands::Export(export) => match export {
            ExportCommand::Weekly { num } => export_weekly(num),
            ExportCommand::WeeklyOpml => export_weekly_opml(),
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

    // Generate blog
    fs::create_dir_all("dist/blog")?;
    for blogpost in &content.blog {
        fs::create_dir_all(format!("dist/blog/{}", blogpost.slug))?;
        let path = format!("dist/blog/{}/index.html", blogpost.slug);
        fs::write(
            &path,
            layout
                .render(templates::blog::render(blogpost)?)
                .into_string(),
        )?;
    }
    // Generate pagination
    let num_pages = content.blog.len() / 8 + 1;
    for (i, blog_posts) in content.blog.chunks(8).enumerate() {
        let page = i + 1;
        fs::create_dir_all(format!("dist/page/{}", page))?;
        let path = format!("dist/page/{}/index.html", page);
        fs::write(
            &path,
            layout
                .render(templates::blog::render_page(page, num_pages, blog_posts)?)
                .into_string(),
        )?;

        // The first page should be the index
        if i == 0 {
            fs::rename("dist/page/1/index.html", "dist/index.html")?;
        }
    }

    // Generate weekly
    fs::create_dir_all("dist/weekly")?;
    fs::write(
        "dist/weekly/index.html",
        layout
            .render(templates::weekly::render_index(&content)?)
            .into_string(),
    )?;
    for weekly_issue in &content.weekly {
        fs::create_dir_all(format!("dist/weekly/{}", weekly_issue.num))?;
        let path = format!("dist/weekly/{}/index.html", weekly_issue.num);
        fs::write(
            &path,
            layout
                .render(templates::weekly::render(weekly_issue)?)
                .into_string(),
        )?;
    }

    // Generate book reviews
    fs::create_dir_all("dist/book-reviews")?;
    fs::write(
        "dist/book-reviews/index.html",
        layout
            .render(templates::book_review::render_index(&content)?)
            .into_string(),
    )?;
    for book_review in &content.book_reviews {
        fs::create_dir_all(format!("dist/book-reviews/{}", book_review.slug))?;
        let path = format!("dist/book-reviews/{}/index.html", book_review.slug);
        fs::write(
            &path,
            layout
                .render(templates::book_review::render(book_review)?)
                .into_string(),
        )?;
    }

    // Generate home screens
    fs::create_dir_all("dist/home-screens")?;
    fs::write(
        "dist/home-screens/index.html",
        layout
            .render(templates::home_screen::render_index(&content)?)
            .into_string(),
    )?;
    for home_screen in &content.home_screens {
        fs::create_dir_all(format!("dist/home-screens/{}", home_screen.slug))?;
        let path = format!("dist/home-screens/{}/index.html", home_screen.slug);
        fs::write(
            &path,
            layout
                .render(templates::home_screen::render(home_screen)?)
                .into_string(),
        )?;
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
            layout.render(templates::page::render(page)?).into_string(),
        )?;
    }

    // Generate projects page
    fs::create_dir_all("dist/projects")?;
    fs::write(
        "dist/projects/index.html",
        layout
            .render(templates::project::render(&content.projects)?)
            .into_string(),
    )?;

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

    let temp_dir = TempDir::new("arne-me-fonts")?;
    let zip_path = temp_dir.path().join("fonts.zip");
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

    temp_dir.close()?;
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

    fs::write(path, content)?;
    Ok(())
}

pub fn new_home_screen() -> Result<()> {
    let now = Utc::now();
    let filename = now.format("%Y-%m");
    let path = Path::new("content")
        .join("home-screens")
        .join(format!("{}.md", filename));
    if path.exists() {
        bail!("Home screen already exists at: {:?}", path);
    }

    let title = format!("My Home Screen in {}", now.format("%B '%y"));
    let description = format!("This is my home screen in {}.", now.format("%B %Y"));
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let assets_path = format!(
        "/home-screens/{}",
        now.format("%B-%Y").to_string().to_lowercase()
    );
    let content = format!(
        r#"---
title: "{title}"
description: "{description}"
location: "TODO"
published: "{date}"
source:
  png: {assets_path}/home-screen.png
  avif: {assets_path}/home-screen.avif
  alt: |
    TODO
---"#
    );

    fs::write(path, content)?;
    Ok(())
}
