use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::Parser;
use std::{env, fs, path::Path, process::Command};

mod og;
mod syndicate;
mod watch;
mod webmentions;

use arneos::content::{Content, Item};
use syndicate::{syndicate_before_sha, syndicate_slug};
use webmentions::send_webmentions;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum NewCommand {
    #[clap(name = "weekly")]
    Weekly,
    #[clap(name = "blog")]
    Blog { slug: String },
    #[clap(name = "book")]
    Book { slug: String },
    #[clap(name = "og-image")]
    OgImage { path: String },
}

#[derive(Debug, Parser)]
enum Commands {
    #[clap(name = "watch")]
    Watch,
    #[clap(name = "send-webmentions")]
    SendWebmentions {
        path: String,
        #[clap(long, short, default_value = "false")]
        dry_run: bool,
    },
    #[command(subcommand)]
    #[clap(name = "new")]
    New(NewCommand),
    #[clap(name = "syndicate")]
    Syndicate {
        #[clap(long, short, group = "subject")]
        before_sha: Option<String>,
        #[clap(long, short, group = "subject")]
        slug: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch => watch::watch(),
        Commands::SendWebmentions { path, dry_run } => send_webmentions(path, dry_run),
        Commands::New(new) => match new {
            NewCommand::Weekly => new_weekly(),
            NewCommand::Blog { slug } => new_blog(slug),
            NewCommand::Book { slug } => new_book(slug),
            NewCommand::OgImage { path } => new_og_image(path),
        },
        Commands::Syndicate {
            before_sha: Some(before_sha),
            slug: _,
        } => syndicate_before_sha(before_sha),
        Commands::Syndicate {
            before_sha: _,
            slug: Some(slug),
        } => syndicate_slug(slug),
        Commands::Syndicate {
            before_sha: None,
            slug: None,
        } => bail!("--before-sha and --slug are exclusive"),
    }
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
    match env::var("EDITOR") {
        Ok(editor) if editor != "" => {
            Command::new(editor).arg(&path).status()?;
        }
        _ => eprintln!("Could not open file: $EDITOR is not set"),
    }
    Ok(())
}

pub fn new_blog(slug: String) -> Result<()> {
    let path = Path::new("content")
        .join("blog")
        .join(format!("{}.md", slug));
    if path.exists() {
        bail!("Blogpost already exists at: {:?}", path);
    }

    let date = Utc::now().format("%Y-%m-%d").to_string();
    let content = format!(
        r#"---
title: ""
description: ""
published: "{date}"
location: ""
---
"#
    );

    fs::write(&path, content)?;
    println!("Created new blogpost at: {:?}", path);
    match env::var("EDITOR") {
        Ok(editor) if editor != "" => {
            Command::new(editor).arg(&path).status()?;
        }
        _ => eprintln!("Could not open file: $EDITOR is not set"),
    }
    Ok(())
}

pub fn new_book(slug: String) -> Result<()> {
    let path = Path::new("content")
        .join("library")
        .join(format!("{}.md", slug));
    if path.exists() {
        bail!("Book already exists at: {:?}", path);
    }

    let date = Utc::now().format("%Y-%m-%d").to_string();
    let content = format!(
        r#"---
title: ""
author: ""
read: "{date}"
rating: 0
location: ""
---
"#
    );

    fs::write(&path, content)?;
    println!("Created new blogpost at: {:?}", path);
    match env::var("EDITOR") {
        Ok(editor) if editor != "" => {
            Command::new(editor).arg(&path).status()?;
        }
        _ => eprintln!("Could not open file: $EDITOR is not set"),
    }
    Ok(())
}

fn new_og_image(path: impl AsRef<str>) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;

    if path.as_ref() == "*" {
        // Regenerate all og images
        content
            .weekly
            .iter()
            .map(|weekly_issue| Ok(new_og_image(format!("weekly/{}", weekly_issue.num))?))
            .collect::<Result<Vec<()>>>()?;
        content
            .blog
            .iter()
            .map(|blog_post| Ok(new_og_image(format!("blog/{}", blog_post.slug))?))
            .collect::<Result<Vec<()>>>()?;
        content
            .library
            .iter()
            .map(|book_review| Ok(new_og_image(format!("library/{}", book_review.slug))?))
            .collect::<Result<Vec<()>>>()?;
        content
            .pages
            .iter()
            .map(|page| Ok(new_og_image(&page.slug)?))
            .collect::<Result<Vec<()>>>()?;

        // Also regen index pages
        og::generate("Arne Bahlo", "static/og-image.png")?;
        og::generate("Arne's Blog", "static/blog/og-image.png")?;
        og::generate("Arne's Weekly", "static/weekly/og-image.png")?;
        og::generate("Arne's Book Reviews", "static/library/og-image.png")?;
    } else {
        match content
            .by_path(path)
            .ok_or(anyhow!("Failed to find item"))?
        {
            Item::Weekly(weekly_issue) => og::generate(
                &weekly_issue.title,
                format!("static/weekly/{}/og-image.png", weekly_issue.num),
            )?,
            Item::Blog(blogpost) => og::generate(
                &blogpost.title,
                format!("static/blog/{}/og-image.png", blogpost.slug),
            )?,
            Item::Book(book) => og::generate(
                &book.title,
                format!("static/library/{}/og-image.png", book.slug),
            )?,
            Item::Page(page) => {
                og::generate(&page.title, format!("static/{}/og-image.png", page.slug))?
            }
        }
    }
    Ok(())
}
