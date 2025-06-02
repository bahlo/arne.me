use anyhow::Result;
use chrono::Utc;
use pichu::Markdown;
use rss::{ChannelBuilder, Item, ItemBuilder};

use crate::{
    blog::Blogpost,
    library::Book,
    weekly::{self, Issue},
};

const RFC_822: &str = "%a, %d %b %Y %T %z";
const RFC_822_DATE: &str = "%a, %d %b %Y 00:00:00 +0000";

pub fn render_blog(blogposts: &Vec<Markdown<Blogpost>>) -> String {
    let items: Vec<Item> = blogposts
        .iter()
        .map(|blogpost| {
            ItemBuilder::default()
                .title(blogpost.frontmatter.title.clone())
                .link(format!("https://arne.me/blog/{}", blogpost.basename))
                .description(blogpost.frontmatter.description.clone())
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/blog/{}", blogpost.basename),
                    permalink: true,
                })
                .pub_date(
                    blogpost
                        .frontmatter
                        .published
                        .format(RFC_822_DATE)
                        .to_string(),
                )
                .content(blogpost.html.clone())
                .build()
        })
        .collect();

    ChannelBuilder::default()
        .title("Arne Bahlo")
        .language(Some("en-us".to_string()))
        .copyright(format!("2021 – {} Arne Bahlo", Utc::now().format("%Y")))
        .managing_editor(Some("hey@arne.me".to_string()))
        .webmaster(Some("hey@arne.me".to_string()))
        .last_build_date(Utc::now().format(RFC_822).to_string())
        .link("https://arne.me")
        .description("Arne Bahlo's personal website")
        .items(items)
        .build()
        .to_string()
}

pub fn render_weekly(issues: &Vec<Markdown<Issue>>) -> Result<String> {
    let items = issues
        .iter()
        .map(|issue| {
            Ok(ItemBuilder::default()
                .title(issue.frontmatter.title.clone())
                .link(format!("https://arne.me/weekly/{}", issue.basename))
                .description(format!("Issue #{} of Arne’s Weekly", issue.basename))
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/weekly/{}", issue.basename),
                    permalink: true,
                })
                .pub_date(issue.frontmatter.date.format(RFC_822_DATE).to_string())
                .content(weekly::render_content(issue, None)?.into_string())
                .build())
        })
        .collect::<Result<Vec<Item>>>()?;

    Ok(ChannelBuilder::default()
        .title("Arne’s Weekly")
        .language(Some("en-us".to_string()))
        .copyright(format!("2021 – {} Arne Bahlo", Utc::now().format("%Y")))
        .managing_editor(Some("hey@arne.me".to_string()))
        .webmaster(Some("hey@arne.me".to_string()))
        .last_build_date(Utc::now().format(RFC_822).to_string())
        .link("https://arne.me/weekly")
        .description("A weekly newsletter with the best stories of the internet.")
        .items(items)
        .build()
        .to_string())
}

pub fn render_library(books: &Vec<Markdown<Book>>) -> String {
    let items: Vec<Item> = books
        .iter()
        .map(|book| {
            ItemBuilder::default()
                .title(format!(
                    "{} by {}",
                    book.frontmatter.title, book.frontmatter.author
                ))
                .link(format!("https://arne.me/library/{}", book.basename))
                .description(format!(
                    "I read {} by {}",
                    book.frontmatter.title, book.frontmatter.author
                ))
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/library/{}", book.basename),
                    permalink: true,
                })
                .pub_date(book.frontmatter.read.format(RFC_822_DATE).to_string())
                .content(book.html.clone())
                .build()
        })
        .collect();

    ChannelBuilder::default()
        .title("Arne Bahlo’s Book Reviews")
        .language(Some("en-us".to_string()))
        .copyright(format!("2021 – {} Arne Bahlo", Utc::now().format("%Y")))
        .managing_editor(Some("hey@arne.me".to_string()))
        .webmaster(Some("hey@arne.me".to_string()))
        .last_build_date(Utc::now().format(RFC_822).to_string())
        .link("https://arne.me/library")
        .description("Every book I read gets a review and ends up here.")
        .items(items)
        .build()
        .to_string()
}
