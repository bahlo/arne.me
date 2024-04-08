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

pub fn render_page(page: usize, num_pages: usize, blog_posts: &[Blogpost]) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
            section.index {
                @for post in blog_posts {
                    div {
                        @let url = format!("/blog/{}", post.slug);
                        h1.blogpost__heading { a href=(url) { (post.title) } }
                        i.blogpost__byline {
                            (format_date(post.published))
                        }
                        @if let Some(excerpt_html) = &post.excerpt_html {
                            p { (PreEscaped(excerpt_html)) }
                            a href=(url) { "Continue reading..." }
                        }
                    }
                }

                br.hidden;

                nav.pagination role="navigation" aria-label="Pagination Navigation" {
                    @for i in 1..=num_pages {
                        @if i == page {
                            span aria-current="true" { (i) }
                        } @else if i == 1 {
                            a href="/" aria-label=(format!("Go to page {}", i)) { (i) }
                        } @else {
                            a href=(format!("/blog/{}", i)) aria-label=(format!("Go to page {}", i)) { (i) }
                        }
                        @if i < num_pages {
                            (PreEscaped(" &nbsp; "))
                        }
                    }
                }
            }
        },
        layout::Options {
            is_index: true,
            ..Default::default()
        },
    ))
}

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
                    i.blogpost__byline {
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
