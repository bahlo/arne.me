use std::collections::HashMap;

use anyhow::Result;
use chrono::{Datelike, Utc};
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{Blogpost, Content},
    templates::{
        format_date,
        layout::{self, Context, Head, OgType},
    },
};

pub fn render_page(content: &Content) -> Result<Context> {
    let mut blog_posts_by_year = content
        .blog
        .iter()
        .fold(HashMap::new(), |mut acc, blogpost| {
            let posts = acc.entry(blogpost.published.year()).or_insert(vec![]);
            posts.push(blogpost);
            acc
        })
        .into_iter()
        .map(|(year, blog_posts)| (year, blog_posts))
        .collect::<Vec<_>>();
    blog_posts_by_year.sort_by(|(a, _), (b, _)| b.cmp(a));

    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
            section.blog {
                h1 { "Blog" }

                @for (year, blog_posts) in blog_posts_by_year {
                    @if year != Utc::now().year() {
                        h2 { (year) }
                    }
                    @for post in blog_posts {
                        div {
                            @let url = format!("/blog/{}", post.slug);
                            a.medium href=(url) { (post.title) }
                            "—"
                            span { (post.description) }
                            br;
                        }
                    }
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::Blog,
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
                .blogpost__header {
                    h1.p-name.blogpost__heading { (blogpost.title) }
                    a.u-url hidden href=(format!("/blog/{}", blogpost.slug)) {}
                    span.p-summary hidden { (blogpost.description) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
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
            navigation_item: layout::NavigationItem::Blog,
        },
    ))
}
