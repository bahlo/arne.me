use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::Parser;
use std::{env, fs, path::Path, process::Command};

mod watch;

use arneos::content::Content;
use automate::{automate_before_sha, automate_path};
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
    #[clap(name = "automate")]
    Automate {
        #[clap(long, short, group = "subject")]
        before_sha: Option<String>,
        #[clap(long, short, group = "subject")]
        path: Option<String>,
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
        },
        Commands::Automate {
            before_sha: Some(before_sha),
            path: _,
        } => automate_before_sha(before_sha),
        Commands::Automate {
            before_sha: _,
            path: Some(path),
        } => automate_path(path),
        Commands::Automate {
            before_sha: None,
            path: None,
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
