use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::layout::{self, format_date, Context, Head, Layout, OgType};

#[derive(Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub title: String,
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toot_of_the_week: Option<WeeklyTootOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tweet_of_the_week: Option<WeeklyTweetOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_of_the_week: Option<WeeklyQuoteOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeet_of_the_week: Option<WeeklySkeetOfTheWeek>,
    #[serde(default)]
    pub categories: Vec<WeeklyCategory>,
    #[serde(rename = "noSubscribeForm")]
    pub no_subscribe_form: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyCategory {
    pub title: String,
    pub stories: Vec<WeeklyStory>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyStory {
    pub title: String,
    pub url: Url,
    pub description: String,
    #[serde(default)]
    pub description_html: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyTootOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklySkeetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyTweetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
    pub media: Option<WeeklyTweetOfTheWeekMedia>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyTweetOfTheWeekMedia {
    pub alt: String,
    pub image: String,
    pub src_set: Vec<SrcSet>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct SrcSet {
    pub src: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct WeeklyQuoteOfTheWeek {
    pub text: String,
    pub author: String,
}

fn subscribe_form() -> Markup {
    html! {
        form.weekly__subscribe_form action="https://buttondown.email/api/emails/embed-subscribe/arnesweekly" method="post" {
            label for="email" { "Email address:" }
            br;
            input required type="email" name="email" id="email" placeholder="you@example.org";
            input type="submit" value="Subscribe";
            br;
            small {
                "Subscription is one click and you can unsubscribe at any time. Your email address will be sent to "
                a href="https://buttondown.email" { "Buttondown" }
                ", the service I use to send out emails."
            }
        }
    }
}

pub fn render_all(layout: &Layout, issues: &Vec<Markdown<Issue>>) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: "Arne’s Weekly".to_string(),
            description: "A weekly newsletter with the best stories of the internet.".to_string(),
            url: Url::parse("https://arne.me/weekly")?,
            og_type: OgType::Website,
        },
        html! {
            section.weekly.h-entry {
                .weekly__header {
                    h1 { "Arne’s Weekly" }
                    p { "A weekly newsletter with the best stories of the internet. There’s an "
                        a href="/weekly/feed.xml" { "RSS Feed" }
                        " available, but you should really subscribe:" }
                    (subscribe_form())
                }
                h2 { "Archive" }
                .weekly__overview {
                    @for issue in issues {
                        li.weekly__overview_issue {
                            a href=(format!("/weekly/{}", issue.basename)) {
                                (issue.frontmatter.title)
                            }
                            .divider {};
                            i.byline {
                                time datetime=(issue.frontmatter.date.format("%Y-%m-%d")) { (format_date(issue.frontmatter.date)) }
                            }
                        }
                    }
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::Newsletter,
            ..Default::default()
        },
    ))
}

#[derive(Debug, Default)]
pub struct RenderOptions {
    pub skip_stories: bool,
}

pub fn render_content(
    weekly: &Markdown<Issue>,
    opts: impl Into<Option<RenderOptions>>,
) -> Result<Markup> {
    let opts = opts.into().unwrap_or_default();
    Ok(html! {
        (PreEscaped(weekly.html.clone()))
        @if let Some(toot_of_the_week) = &weekly.frontmatter.toot_of_the_week {
            h2 { "Toot of the Week" }
            blockquote {
                (toot_of_the_week.text)
                (PreEscaped("&mdash;&nbsp;"))
                a href=(toot_of_the_week.url) {
                    (toot_of_the_week.author)
                }
            }
        }
        @if let Some(tweet_of_the_week) = &weekly.frontmatter.tweet_of_the_week {
            h2 { "Tweet of the Week" }
            blockquote {
                (tweet_of_the_week.text)
                @if let Some(media) = &tweet_of_the_week.media {
                    @if media.src_set.len() > 0 {
                        picture {
                            @for source in &media.src_set {
                                source srcset=(source.src) type=(source.typ);
                            }
                            img src=(media.image) alt=(media.alt);
                        }
                    } @else {
                        img src=(media.image) alt=(media.alt);
                    }
                }
                (PreEscaped("&mdash;&nbsp;"))
                a href=(tweet_of_the_week.url) {
                    (tweet_of_the_week.author)
                }
            }
        }
        @if let Some(quote_of_the_week) = &weekly.frontmatter.quote_of_the_week {
            h2 { "Quote of the Week" }
            blockquote {
                "“"
                (quote_of_the_week.text.trim())
                "” "
                (PreEscaped("&mdash;"))
                (quote_of_the_week.author)
            }
        }
        @if let Some(skeet_of_the_week) = &weekly.frontmatter.skeet_of_the_week {
            h2 { "Skeet of the Week" }
            blockquote {
                (skeet_of_the_week.text)
                (PreEscaped("&mdash;&nbsp;"))
                a href=(skeet_of_the_week.url) {
                    (skeet_of_the_week.author)
                }
            }
        }
        @if !opts.skip_stories {
            @for category in &weekly.frontmatter.categories {
                h2 { (category.title) }
                ul {
                    @for story in &category.stories {
                        @let host = story.url.host_str().ok_or(anyhow!("Failed to get host for {} in weekly issue #{}", story.url, weekly.basename))?;

                        li {
                            a href=(story.url) {
                                (story.title)
                            }
                            span.weekly__url { (format!(" ({})", host.strip_prefix("www.").unwrap_or(host))) }
                            (PreEscaped(story.description_html.clone()))
                        }
                    }
                }
            }
        }
    })
}

pub fn render_single(layout: &Layout, issue: &Markdown<Issue>) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: issue.frontmatter.title.clone(),
            description: format!("Arne's Weekly #{}", issue.basename),
            url: Url::parse(&format!("https://arne.me/weekly/{}", issue.basename))?,
            og_type: OgType::Article,
        },
        html! {
            article.weekly.h-entry {
                div {
                    h1.p-name.weekly__heading { (issue.frontmatter.title) }
                    a.u-url hidden href=(format!("/weekly/{}", issue.basename)) {}
                    span.p-summary hidden { (format!("Arne's Weekly #{}", issue.basename)) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
                        time.dt-published datetime=(issue.frontmatter.date.format("%Y-%m-%d")) { (format_date(issue.frontmatter.date)) }
                    }
                }
                .e-content {
                    (render_content(issue, None)?)
                }
                @if !issue.frontmatter.no_subscribe_form.unwrap_or_default() {
                    h2 { "Subscribe" }
                    p { "Get Arne's Weekly in your inbox every Sunday. No ads, no shenanigans. I do sometimes feature projects of friends."}
                    (subscribe_form())
                }
            }
        },
        layout::Options {
            navigation_item: layout::NavigationItem::Newsletter,
            source_path: Some(format!("content/weekly/{}.md", issue.basename)),
        },
    ))
}
