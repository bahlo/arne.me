use std::collections::HashMap;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::Deserialize;
use url::Url;

use crate::layout::{self, format_date, Context, Head, Layout, OgType};

#[derive(Debug, Deserialize)]
pub struct Blogpost {
    pub title: String,
    pub description: String,
    pub location: String,
    pub updated: Option<NaiveDate>,
    pub published: NaiveDate,
    #[serde(default)]
    pub hidden: bool,
    pub hackernews: Option<Url>,
    pub lobsters: Option<Url>,
}

pub fn render_all(layout: &Layout, blog_posts: &Vec<Markdown<Blogpost>>) -> Result<Markup> {
    let mut blog_posts_by_year = blog_posts
        .iter()
        .filter(|blog_post| !blog_post.frontmatter.hidden)
        .fold(HashMap::new(), |mut acc, blogpost| {
            let posts = acc
                .entry(blogpost.frontmatter.published.year())
                .or_insert(vec![]);
            posts.push(blogpost);
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();
    blog_posts_by_year.sort_by(|(a, _), (b, _)| b.cmp(a));

    layout.render(Context::new_with_options(
        Head {
            title: "Arne's Blog".to_string(),
            description: "Arne Bahlo's personal blog".to_string(),
            url: Url::parse("https://arne.me/blog")?,
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
                            @let url = format!("/blog/{}", post.basename);
                            a.medium href=(url) { (post.frontmatter.title) }
                            "—"
                            span { (post.frontmatter.description) }
                            br;
                        }
                    }
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::Blog,
            ..Default::default()
        },
    ))
}

pub fn render_single(layout: &Layout, blog_post: &pichu::Markdown<Blogpost>) -> Result<Markup> {
    layout.render(Context::new_with_options(
            Head {
                title: blog_post.frontmatter.title.clone(),
                description: blog_post.frontmatter.description.clone(),
                url: Url::parse(&format!("https://arne.me/blog/{}", blog_post.basename))?,
                og_type: OgType::Article,
            },
            html! {
                article.blogpost.h-entry {
                    .blogpost__header {
                        h1.p-name.blogpost__heading { (blog_post.frontmatter.title) }
                        a.u-url hidden href=(format!("/blog/{}", blog_post.basename)) {}
                        span.p-summary hidden { (blog_post.frontmatter.description) }
                        span.p-author hidden { "Arne Bahlo" }
                        i.byline {
                            time.dt-published datetime=(blog_post.frontmatter.published.format("%Y-%m-%d")) { (format_date(blog_post.frontmatter.published)) }
                            (PreEscaped(" &middot; "))
                            span.p-location { (blog_post.frontmatter.location) }
                            @if blog_post.frontmatter.hackernews.is_some() || blog_post.frontmatter.lobsters.is_some() {
                                (PreEscaped(" &middot; "))
                                @if let Some(hackernews) = &blog_post.frontmatter.hackernews {
                                    a href=(hackernews) { "HN" }
                                    @if blog_post.frontmatter.lobsters.is_some() {
                                        ", "
                                    }
                                }
                                @if let Some(lobsters) = &blog_post.frontmatter.lobsters {
                                    a href=(lobsters) { "Lobsters" }
                                }
                            }

                        }
                    }
                    .e-content {
                        (PreEscaped(blog_post.html.clone()))
                    }
                }
            },
            layout::Options {
                navigation_item: layout::NavigationItem::Blog,
                source_path: Some(format!("content/blog/{}.md", blog_post.basename)),
            },
        ))
}
