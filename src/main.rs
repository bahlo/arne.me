use anyhow::{anyhow, bail, Result};
use clap::Parser;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::Path,
    process::Command,
};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use tempdir::TempDir;
use templates::layout::Layout;
use zip::ZipArchive;

mod content;
mod rss;
mod sitemap;
mod templates;
#[cfg(feature = "watch")]
mod watch;
#[cfg(feature = "send-webmentions")]
mod webmentions;

use crate::content::Content;
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
enum Commands {
    #[clap(name = "build")]
    Build {
        #[clap(long)]
        websocket_port: Option<u16>,
    },
    #[cfg(feature = "watch")]
    #[clap(name = "watch")]
    Watch,
    #[clap(name = "export-weekly")]
    ExportWeekly { num: u16 },
    #[clap(name = "download-fonts")]
    DownloadFonts,
    #[cfg(feature = "send-webmentions")]
    #[clap(name = "send-webmentions")]
    SendWebmentions {
        path: String,
        #[clap(long, short, default_value = "false")]
        dry_run: bool,
    },
    #[clap(name = "compile-syntax")]
    CompileSyntax,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { websocket_port } => build(websocket_port),
        #[cfg(feature = "watch")]
        Commands::Watch => watch::watch(),
        Commands::ExportWeekly { num } => export_weekly(num),
        Commands::DownloadFonts => download_fonts(),
        #[cfg(feature = "send-webmentions")]
        Commands::SendWebmentions { path, dry_run } => send_webmentions(path, dry_run),
        Commands::CompileSyntax => compile_syntax(),
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
    fs::write(
        "dist/index.html",
        layout
            .render(templates::index::render(&content)?)
            .into_string(),
    )?;

    // Generate articles
    fs::create_dir_all("dist/articles")?;
    fs::write(
        "dist/articles/index.html",
        layout
            .render(templates::article::render_index(&content)?)
            .into_string(),
    )?;
    for article in &content.articles {
        fs::create_dir_all(format!("dist/articles/{}", article.slug))?;
        let path = format!("dist/articles/{}/index.html", article.slug);
        fs::write(
            &path,
            layout
                .render(templates::article::render(article)?)
                .into_string(),
        )?;
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
    fs::write("dist/feed.xml", rss::render_articles(&content))?;
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

fn export_weekly(num: u16) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;
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

fn syntax_definition_from_path(
    path: impl AsRef<Path>,
    lang: impl AsRef<str>,
) -> Result<SyntaxDefinition> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let syntax_string = String::from_utf8(buffer)?;
    let syntax = SyntaxDefinition::load_from_str(&syntax_string, true, Some(lang.as_ref()))?;
    Ok(syntax)
}

fn compile_syntax() -> Result<()> {
    let mut ps_builder = SyntaxSet::load_defaults_newlines().into_builder();

    let ts_syntax = syntax_definition_from_path("syntax/TypeScript.sublime-syntax", "typescript")?;
    ps_builder.add(ts_syntax);

    let viml_syntax = syntax_definition_from_path("syntax/viml.sublime-syntax", "vim")?;
    ps_builder.add(viml_syntax);

    let nix_syntax = syntax_definition_from_path("syntax/nix.sublime-syntax", "nix")?;
    ps_builder.add(nix_syntax);

    let syntax_set = ps_builder.build();

    let bytes = bincode::serialize(&syntax_set)?;
    let file = File::create("syntax/syntax_set")?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&bytes)?;

    Ok(())
}
