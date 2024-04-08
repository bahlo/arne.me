use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::Blogpost,
    templates::{
        format_date,
        layout::{self, Context, Head, OgType},
    },
};

pub fn render(blogpost: &Blogpost) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: blogpost.title.clone(),
            description: blogpost.description.clone(),
            url: Url::parse(&format!("https://arne.me/blog/{}", blogpost.slug))?,
            og_type: OgType::Article,
        },
        html! {
            article.blogpost.h-entry {
                header.blogpost__header {
                    h1.p-name.blogpost__heading { (blogpost.title) }
                    a.u-url hidden href=(format!("/blog/{}", blogpost.slug)) {}
                    span.p-summary hidden { (blogpost.description) }
                    span.p-author hidden { "Arne Bahlo" }
                    em.blogpost__byline {
                        time.dt-published datetime=(blogpost.published.format("%Y-%m-%d")) { (format_date(blogpost.published)) }
                        (PreEscaped(" &middot; "))
                        span.p-location { (blogpost.location) }
                        @if blogpost.hackernews.is_some() || blogpost.lobsters.is_some() {
                            (PreEscaped(" &middot; "))
                            @if let Some(hackernews) = &blogpost.hackernews {
                                a href=(hackernews) { "HN" }
                                @if blogpost.lobsters.is_some() {
                                    ", "
                                }
                            }
                            @if let Some(lobsters) = &blogpost.lobsters {
                                a href=(lobsters) { "Lobsters" }
                            }
                        }

                    }
                }
                .e-content {
                    (PreEscaped(blogpost.content_html.clone()))
                }
            }
        },
        layout::Options {
            back_link: Some("/blog".to_string()),
            ..Default::default()
        },
    ))
}
