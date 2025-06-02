use anyhow::{anyhow, Result};
use maud::{html, Markup};
use pichu::Markdown;
use url::Url;

use crate::{
    blog::Blogpost,
    layout::{self, Context, Head, Layout, OgType},
    library::Book,
    weekly::Issue,
};

pub fn render(
    layout: &Layout,
    blog: &Vec<Markdown<Blogpost>>,
    weekly: &Vec<Markdown<Issue>>,
    books: &Vec<Markdown<Book>>,
) -> Result<Markup> {
    let last_blogpost = blog.first().ok_or(anyhow!("No blogposts found"))?;
    let last_weekly = weekly.first().ok_or(anyhow!("No weekly issues found"))?;
    let last_book = books.first().ok_or(anyhow!("No books found"))?;
    layout.render(Context::new_with_options(
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
              a href=(format!("/blog/{}", last_blogpost.basename)) { (last_blogpost.frontmatter.title) }
              ", the last newsletter issue is "
              a href=(format!("/weekly/{}", last_weekly.basename)) { (last_weekly.frontmatter.title) }
              " and the last book I've read is "
              a href=(format!("/library/{}", last_book.basename)) {
                  (last_book.frontmatter.title)
                  " by "
                  (last_book.frontmatter.author)
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
