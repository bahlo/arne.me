use anyhow::Result;
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::Deserialize;
use url::Url;

use crate::templates::{
    format_date,
    layout::{self, Context, Head, Layout, OgType},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub read: NaiveDate,
    pub rating: u8,
    pub location: String,
    pub url: Option<Url>,
}

pub fn render_single(layout: &Layout, book: &Markdown<Book>) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: format!("I read {} by {}", book.frontmatter.title, book.frontmatter.author),
            description: format!("I read {} by {}", book.frontmatter.title, book.frontmatter.author),
            url: Url::parse(&format!("https://arne.me/library/{}", book.basename))?,
            og_type: OgType::Article,
        },
        html! {
            article.book_reviews.h-entry {
                div {
                    h1.p-name { (book.frontmatter.title) " by " (book.frontmatter.author) }
                    a.u-url hidden { (format!("/library/{}", book.basename)) }
                    span.p-summary hidden { (format!("I read {} by {}", book.frontmatter.title, book.frontmatter.author)) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
                        time.dt-published datetime=(book.frontmatter.read.format("%Y-%m-%d")) { (format_date(book.frontmatter.read)) }
                        (PreEscaped(" &middot; "))
                        span.p-location { (book.frontmatter.location) }
                        (PreEscaped(" &middot; "))
                        (book.frontmatter.rating)
                        "/5"
                        @if let Some(url) = &book.frontmatter.url {
                            (PreEscaped(" &middot; "))
                            a href=(url) { "Buy" }
                        }
                    }
                }
                picture {
                    source srcset=(format!("/library/{}/cover.avif", book.basename)) type="image/avif";
                    img.book_reviews__cover src=(format!("/library/{}/cover.jpg", book.basename)) alt=(format!("The cover of {} by {}", book.frontmatter.title, book.frontmatter.author));
                }
                .e-content {
                    (PreEscaped(book.html.clone()))
                }
                .clear;
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::BookReviews,
            source_path: Some(format!("content/library/{}.md", book.basename)),
        },
    ))
}

pub fn render_all(layout: &Layout, books: &Vec<Markdown<Book>>) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: "Arne's Library".to_string(),
            description: "Every book I read gets a review and ends up here.".to_string(),
            url: Url::parse("https://arne.me/library")?,
            og_type: OgType::Website,
        },
        html! {
            section.book_reviews {
                h1 { "Library" }
                p {
                    "Since March 2023 I'm writing book reviews for all books I'm reading:"
                }
                .book_reviews__grid {
                    @for book in books {
                        a href=(format!("/library/{}", book.basename)) {
                            picture {
                                source srcset=(format!("/library/{}/cover-small.avif", book.basename)) type="image/avif";
                                img width="100" src=(format!("/library/{}/cover-small.jpg", book.basename)) alt=(format!("The cover of {} by {}", book.frontmatter.title, book.frontmatter.author));
                            }
                        }
                    }
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::BookReviews,
            ..Default::default()
        },
    ))
}
