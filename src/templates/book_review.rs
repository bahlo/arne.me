use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::{BookReview, Content},
    layout::{self, Head, OgType},
    templates::format_date,
};

pub fn render(book_review: &BookReview, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: &format!(
                "Book Review: {} by {}",
                book_review.title, book_review.author
            ),
            description: &format!("I read {} by {}", book_review.title, book_review.author,),
            url: Url::parse(&format!("https://arne.me/book-review/{}", book_review.slug))?,
            og_type: OgType::Article,
            css_hash: css_hash.as_ref(),
        },
        html! {
            article.article {
                header {
                    h1 { (book_review.title) " by " (book_review.author) }
                    em.article__byline {
                        "Read on "
                        time datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                        " in " (book_review.location) ", rated " (book_review.rating) "/5"
                    }
                }
                (PreEscaped(book_review.content_html.clone()))
            }
        },
        None,
    ))
}

pub fn render_index(content: &Content, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Book Reviews",
            description: "Every book I read gets a review and ends up here.",
            url: Url::parse("https://arne.me/book-reviews")?,
            og_type: OgType::Website,
            css_hash: css_hash.as_ref(),
        },
        html! {
            h1 { "Book reviews" }
            @for book_review in &content.book_reviews {
                article.article {
                    header {
                        h2 {
                            a href=(format!("/book-reviews/{}", book_review.slug)) {
                                (book_review.title) " by " (book_review.author)
                            }
                        }
                        em.article__byline {
                            "Read on on "
                            time datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                            " in " (book_review.location) ", rated " (book_review.rating) "/5"
                        }
                    }
                    (PreEscaped(book_review.excerpt_html.clone()))
                    p {
                        a href=(format!("/book-reviews/{}", book_review.slug)) {
                            "Read more" (PreEscaped("&hellip;"))
                        }
                    }
                }
            }
        },
        None,
    ))
}
