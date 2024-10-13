use anyhow::Result;
use chrono::Utc;
use rss::{ChannelBuilder, Item, ItemBuilder};

use crate::templates;
use arneos::content::Content;

const RFC_822: &str = "%a, %d %b %Y %T %z";
const RFC_822_DATE: &str = "%a, %d %b %Y 00:00:00 +0000";

pub fn render_blog(content: &Content) -> String {
    let items: Vec<Item> = content
        .blog
        .iter()
        .map(|blogpost| {
            ItemBuilder::default()
                .title(blogpost.title.clone())
                .link(format!("https://arne.me/blog/{}", blogpost.slug))
                .description(blogpost.description.clone())
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/blog/{}", blogpost.slug),
                    permalink: true,
                })
                .pub_date(blogpost.published.format(RFC_822_DATE).to_string())
                .content(blogpost.content_html.clone())
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

pub fn render_weekly(content: &Content) -> Result<String> {
    let items = content
        .weekly
        .iter()
        .map(|weekly_issue| {
            Ok(ItemBuilder::default()
                .title(weekly_issue.title.clone())
                .link(format!("https://arne.me/weekly/{}", weekly_issue.num))
                .description(format!("Issue #{} of Arne’s Weekly", weekly_issue.num))
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/weekly/{}", weekly_issue.num),
                    permalink: true,
                })
                .pub_date(weekly_issue.published.format(RFC_822_DATE).to_string())
                .content(templates::weekly::render_content(weekly_issue, None)?.into_string())
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

pub fn render_library(content: &Content) -> String {
    let items: Vec<Item> = content
        .library
        .iter()
        .map(|book| {
            ItemBuilder::default()
                .title(format!("{} by {}", book.title, book.author))
                .link(format!("https://arne.me/library/{}", book.slug))
                .description(format!("I read {} by {}", book.title, book.author))
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/library/{}", book.slug),
                    permalink: true,
                })
                .pub_date(book.read.format(RFC_822_DATE).to_string())
                .content(book.content_html.clone())
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
