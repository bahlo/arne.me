use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{BookReview, Content},
    templates::{
        format_date,
        layout::{self, Head, OgType},
    },
};

use super::layout::Context;

pub fn render(book_review: &BookReview) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: format!(
                "Book Review: {} by {}",
                book_review.title, book_review.author
            ),
            description: format!("I read {} by {}", book_review.title, book_review.author),
            url: Url::parse(&format!(
                "https://arne.me/book-reviews/{}",
                book_review.slug
            ))?,
            og_type: OgType::Article,
        },
        html! {
            article.book_reviews.h-entry {
                div {
                    h1.p-name { (book_review.title) " by " (book_review.author) }
                    a.u-url hidden { (format!("/book-reviews/{}", book_review.slug)) }
                    span.p-summary hidden { (format!("I read {} by {}", book_review.title, book_review.author)) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
                        time.dt-published datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                        (PreEscaped(" &middot; "))
                        span.p-location { (book_review.location) }
                        (PreEscaped(" &middot; "))
                        (book_review.rating)
                        "/5"
                    }
                }
                picture {
                    source srcset=(format!("/book-reviews/{}/cover.avif", book_review.slug)) type="image/avif";
                    img.book_reviews__cover src=(format!("/book-reviews/{}/cover.jpg", book_review.slug)) alt=(format!("The cover of {} by {}", book_review.title, book_review.author));
                }
                .e-content {
                    (PreEscaped(book_review.content_html.clone()))
                }
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
            title: "Book Reviews".to_string(),
            description: "Every book I read gets a review and ends up here.".to_string(),
            url: Url::parse("https://arne.me/book-reviews")?,
            og_type: OgType::Website,
        },
        html! {
            section.book_reviews {
                h1 { "Book reviews" }
                p {
                    "Since March 2023 I'm writing book reviews for all books I'm reading:"
                }
                .book_reviews__grid {
                    @for book_review in &content.book_reviews {
                        a href=(format!("/book-reviews/{}", book_review.slug)) {
                            picture {
                                source srcset=(format!("/book-reviews/{}/cover-small.avif", book_review.slug)) type="image/avif";
                                img width="100" src=(format!("/book-reviews/{}/cover-small.jpg", book_review.slug)) alt=(format!("The cover of {} by {}", book_review.title, book_review.author));
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
