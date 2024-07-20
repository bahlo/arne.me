use anyhow::{anyhow, Result};
use maud::html;
use url::Url;

use crate::{
    content::Content,
    templates::layout::{self, Context, Head, OgType},
};

pub fn render(content: &Content) -> Result<Context> {
    let latest_blogpost = content.blog.first().ok_or(anyhow!("No blogposts found"))?;
    let latest_weekly = content
        .weekly
        .first()
        .ok_or(anyhow!("No weekly issues found"))?;
    let latest_book_review = content
        .book_reviews
        .first()
        .ok_or(anyhow!("No book reviews found"))?;
    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".into(),
            description: "This is my personal website.".into(),
            url: Url::parse("https://arne.me/")?,
            og_type: OgType::Website,
        },
        html! {
          section.index {
            .index__hero {
              h1 { "Hej, I'm Arne—" }
              big { "a developer from Frankfurt, Germany" }
            }
            p {
              "You can find me in the Fediverse as "
              a href="https://spezi.social/@arne" { "@arne@spezi.social" }
              " or send me an email at "
              a href="mailto:hey@arne.me" { "hey@arne.me" }
              "."
            }
            p {
              "The latest blog post is titled "
              a href=(format!("/blog/{}", latest_blogpost.slug)) { (latest_blogpost.title) }
              ", the latest newsletter issue is "
              a href=(format!("/weekly/{}", latest_weekly.num)) { (latest_weekly.title) }
              " and the latest book I've read is called "
              a href=(format!("/book-reviews/{}", latest_book_review.slug)) { (latest_book_review.title) }
              "."
            }
          }
        },
        layout::Options::default(),
    ))
}
