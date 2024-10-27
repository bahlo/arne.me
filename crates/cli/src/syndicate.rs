use anyhow::{bail, Result};
use git2::{Delta, DiffDelta, Oid, Repository};
use std::{fs, thread::sleep, time::Duration};

use arneos::content::Content;

fn syndicate_diff_cb(diff_delta: DiffDelta<'_>, _i: f32) -> bool {
    if diff_delta.status() != Delta::Added {
        return true; // continue
    }

    let filepath = match diff_delta.new_file().path() {
        Some(filepath) => filepath,
        None => {
            eprintln!(
                "Failed to get the path of one of the new files: {:?}",
                diff_delta.new_file()
            );
            return true;
        }
    };

    if filepath.extension().and_then(|s| s.to_str()) != Some("md") {
        // Not a markdown file, "continue"
        return true;
    }

    let slug = match filepath
        .strip_prefix("content/")
        .map(|path| path.with_extension(""))
    {
        Ok(slug) => slug,
        Err(e) => {
            eprintln!("Failed to strip the 'content/' prefix: {e}");
            return true;
        }
    };

    if let Err(e) = syndicate_path(slug.to_string_lossy()) {
        eprintln!("Failed to syndicate {slug:?}: {e}");
    };

    true // continue
}

pub fn syndicate_before_sha(before_sha: String) -> Result<()> {
    let repo = Repository::open(".")?;

    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;

    let before_commit_oid = Oid::from_str(&before_sha)?;
    let before_commit = repo.find_commit(before_commit_oid)?;
    let before_commit_tree = before_commit.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&before_commit_tree), Some(&head_tree), None)?;

    diff.foreach(&mut syndicate_diff_cb, None, None, None)?;

    Ok(())
}

pub fn wait_for_200(slug: impl AsRef<str>) -> Result<()> {
    let url = format!("https://arne.me/{}", slug.as_ref());
    println!("Waiting for {url} to return HTTP 200");
    for i in 0..300 {
        // 5 mins
        match ureq::get(&url).call() {
            Ok(res) => {
                if res.status() == 200 {
                    return Ok(());
                } else {
                    eprintln!("Received HTTP {}, retrying in 1s ({i}/300)", res.status());
                }
            }
            Err(e) => {
                eprintln!("Request failed: {e}, retrying in 1s ({i}/300)");
            }
        };
        sleep(Duration::from_secs(1))
    }

    bail!("Failed to reach {url} in 5 minutes")
}

pub fn syndicate_path(slug: impl Into<String>) -> Result<()> {
    let path = slug.into();

    println!("Syndicating {path}...");
    wait_for_200(&path)?;

    let content = Content::parse(fs::read_dir("content")?)?;
    match content.by_path(path) {
        Some(arneos::content::Item::Weekly(weekly_issue)) => {
            let num = weekly_issue.num;
            println!("Would toot this:");
            println!("  📬 Arne’s Weekly #{num} has been sent out, check your inbox or read it online at https://arne.me/weekly/{num}");
        }
        Some(arneos::content::Item::Blog(blogpost)) => {
            let title = &blogpost.title;
            let slug = &blogpost.slug;
            println!("Would toot this:");
            println!("  📝 {title} https://arne.me/blog/{slug}");
        }
        Some(arneos::content::Item::Book(book)) => {
            let slug = &book.slug;
            let title = &book.title;
            let author = &book.author;
            println!("Would toot this:");
            println!("  📚 I read {title} by {author}: https://arne.me/library/{slug}");
        }
        _ => eprintln!("Syndicating weekly issues and blog posts only"),
    }

    Ok(())
}
