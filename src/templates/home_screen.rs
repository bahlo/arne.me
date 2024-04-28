use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{Content, HomeScreen},
    templates::{
        format_date,
        layout::{self, Context, Head, OgType},
    },
};

pub fn render(home_screen: &HomeScreen) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: home_screen.title.clone(),
            description: home_screen.description.clone(),
            url: Url::parse(&format!(
                "https://arne.me/home-screens/{}",
                home_screen.slug
            ))?,
            og_type: OgType::Article,
        },
        html! {
            article.blogpost.h-entry {
                header.blogpost__header {
                    h1.p-name.blogpost__heading { (home_screen.title) }
                    a.u-url hidden href=(format!("/home-screens/{}", home_screen.slug)) {}
                    span.p-summary hidden { (home_screen.description) }
                    span.p-author hidden { "Arne Bahlo" }
                    i.byline {
                        time.dt-published datetime=(home_screen.published.format("%Y-%m-%d")) { (format_date(home_screen.published)) }
                        (PreEscaped(" &middot; "))
                        span.p-location { (home_screen.location) }
                    }
                }
                .e-content {
                    (PreEscaped(home_screen.content_html.clone()))
                }
            }
        },
        layout::Options {
            back_link: Some("/home-screens".to_string()),
            ..Default::default()
        },
    ))
}

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: "Home Screens".to_string(),
            description: "Home Screens by Arne Bahlo.".to_string(),
            url: Url::parse("https://arne.me/home-screens")?,
            og_type: OgType::Website,
        },
        html! {
            section.page {
                h1 { "Home Screens" }
                @for home_screen in content.home_screens.iter() {
                    div {
                        h3.blogpost__heading { a href=(format!("/home-screens/{}", home_screen.slug)) { (home_screen.title) } }
                        i.byline {
                            time datetime=(home_screen.published.format("%Y-%m-%d")) {(format_date(home_screen.published))}
                            (PreEscaped(" &middot; "))
                            (home_screen.location)
                        }
                    }
                }
            }
        },
    ))
}
