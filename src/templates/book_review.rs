use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{BookReview, Content},
    templates::{
        format_date,
        layout::{Head, OgType},
    },
};

use super::layout::Context;

pub fn render(book_review: &BookReview) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: format!(
                "Book Review: {} by {}",
                book_review.title, book_review.author
            ),
            description: format!("I read {} by {}", book_review.title, book_review.author,),
            url: Url::parse(&format!(
                "https://arne.me/book-reviews/{}",
                book_review.slug
            ))?,
            og_type: OgType::Article,
        },
        html! {
            article.book_review {
                header {
                    h1 { (book_review.title) " by " (book_review.author) }
                    em.article_review__byline {
                        "Read on "
                        time datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                        " in " (book_review.location) ", rated " (book_review.rating) "/5"
                    }
                }
                picture {
                    source srcset=(format!("/book-reviews/{}/cover.avif", book_review.slug)) type="image/avif";
                    img.book_review__cover src=(format!("/book-reviews/{}/cover.jpg", book_review.slug)) alt=(format!("The cover of {} by {}", book_review.title, book_review.author));
                }
                (PreEscaped(book_review.content_html.clone()))
            }
        },
    ))
}

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: "Book Reviews".to_string(),
            description: "Every book I read gets a review and ends up here.".to_string(),
            url: Url::parse("https://arne.me/book-reviews")?,
            og_type: OgType::Website,
        },
        html! {
            section.book_reviews {
                h1 { "Book reviews" }
                @for book_review in &content.book_reviews {
                    div {
                        h2.inheritFontSize  {
                                a href=(format!("/book-reviews/{}", book_review.slug)) {
                                    (book_review.title) " by " (book_review.author)
                                }
                        }
                        em.article__byline {
                            time datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                            (PreEscaped(" &middot; "))
                            (book_review.location)
                            (PreEscaped(" &middot; "))
                            (book_review.rating)
                            "/5"
                        }
                    }
                }
            }
        },
    ))
}
