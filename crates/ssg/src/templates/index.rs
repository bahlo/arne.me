use anyhow::{anyhow, Result};
use maud::html;
use url::Url;

use crate::templates::layout::{self, Context, Head, OgType};
use arneos::content::Content;

pub fn render(content: &Content) -> Result<Context> {
    let last_blogpost = content.blog.first().ok_or(anyhow!("No blogposts found"))?;
    let last_weekly = content
        .weekly
        .first()
        .ok_or(anyhow!("No weekly issues found"))?;
    let last_book = content.library.first().ok_or(anyhow!("No books found"))?;
    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".into(),
            description: "This is my personal website.".into(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
          section.index {
            .index__hero {
              h1 { "Hej, I'm Arne—" }
              big { "a developer based in Kiel, Germany." }
            }
            p {
              "This is my home on the web.
              I write code at "
              a href="https://axiom.co" { "Axiom" }
              " for a living.
              Outside of software I like to read, ride my bike, run or swim
              and spend time with my family. You can find me in the Fediverse as "
              a href="https://spezi.social/@arne" { "@arne@spezi.social" }
              " or drop me an email at "
              a href="mailto:hey@arne.me" { "hey@arne.me" }
              ".
              "
            }
            p {
              "The last blog post is titled "
              a href=(format!("/blog/{}", last_blogpost.slug)) { (last_blogpost.title) }
              ", the last newsletter issue is "
              a href=(format!("/weekly/{}", last_weekly.num)) { (last_weekly.title) }
              " and the last book I've read is "
              a href=(format!("/library/{}", last_book.slug)) {
                  (last_book.title)
                  " by "
                  (last_book.author)
              }
              "."
            }
          }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::Home,
            source_path: Some(format!("crates/ssg/src/templates/index.rs")),
        },
    ))
}
