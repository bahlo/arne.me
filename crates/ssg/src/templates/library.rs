use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::templates::{
    format_date,
    layout::{self, Head, OgType},
};
use arneos::content::{Book, Content};

use super::layout::Context;

pub fn render(book: &Book) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: format!("I read {} by {}", book.title, book.author),
            description: format!("I read {} by {}", book.title, book.author),
            url: Url::parse(&format!("https://arne.me/library/{}", book.slug))?,
            og_type: OgType::Article,
        },
        html! {
            article.book_reviews.h-entry {
                div {
                    h1.p-name { (book.title) " by " (book.author) }
                    a.u-url hidden { (format!("/library/{}", book.slug)) }
                    span.p-summary hidden { (format!("I read {} by {}", book.title, book.author)) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
                        time.dt-published datetime=(book.read.format("%Y-%m-%d")) { (format_date(book.read)) }
                        (PreEscaped(" &middot; "))
                        span.p-location { (book.location) }
                        (PreEscaped(" &middot; "))
                        (book.rating)
                        "/5"
                    }
                }
                picture {
                    source srcset=(format!("/library/{}/cover.avif", book.slug)) type="image/avif";
                    img.book_reviews__cover src=(format!("/library/{}/cover.jpg", book.slug)) alt=(format!("The cover of {} by {}", book.title, book.author));
                }
                .e-content {
                    (PreEscaped(book.content_html.clone()))
                }
                .clear;
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::BookReviews,
        },
    ))
}

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new_with_options(
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
                    @for book in &content.library {
                        a href=(format!("/library/{}", book.slug)) {
                            picture {
                                source srcset=(format!("/library/{}/cover-small.avif", book.slug)) type="image/avif";
                                img width="100" src=(format!("/library/{}/cover-small.jpg", book.slug)) alt=(format!("The cover of {} by {}", book.title, book.author));
                            }
                        }
                    }
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::BookReviews,
        },
    ))
}
