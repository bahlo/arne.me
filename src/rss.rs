use chrono::Utc;
use rss::{ChannelBuilder, Item, ItemBuilder};

use crate::content::Content;

const RFC_822: &str = "%a, %d %b %Y %T %z";
const RFC_822_DATE: &str = "%a, %d %b %Y 00:00:00 UT";

pub fn render_feed(content: &Content) -> String {
    let items: Vec<Item> = content
        .articles
        .iter()
        .map(|article| {
            ItemBuilder::default()
                .title(article.title.clone())
                .link(format!("https://arne.me/articles/{}", article.slug))
                .description(article.description.clone())
                .author("Arne Bahlo".to_string())
                .guid(rss::Guid {
                    value: format!("https://arne.me/articles/{}", article.slug),
                    permalink: true,
                })
                .pub_date(article.published.format(RFC_822_DATE).to_string())
                .content(article.content_html.clone())
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
