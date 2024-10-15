use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::Parser;
use std::{fs, path::Path};

mod og;
mod watch;
mod webmentions;

use arneos::content::{Content, Item};
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
enum ExportCommand {
    #[clap(name = "weekly")]
    Weekly { num: Option<u16> },
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
    #[command(subcommand)]
    #[clap(name = "export")]
    Export(ExportCommand),
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
        Commands::Export(export) => match export {
            ExportCommand::Weekly { num } => export_weekly(num),
        },
    }
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
            println!("{}", story.url.host().unwrap());
            println!();
            println!("{}", story.description);
        });
        println!();
    });

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
