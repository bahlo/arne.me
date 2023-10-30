use chrono::Utc;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::fmt::Display;
use url::Url;

#[derive(Debug)]
pub struct Head {
    pub title: String,
    pub description: String,
    pub url: Url,
    pub og_type: OgType,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub enum OgType {
    #[default]
    Website,
    Article,
    Product,
}

impl Display for OgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OgType::Website => write!(f, "website"),
            OgType::Article => write!(f, "article"),
            OgType::Product => write!(f, "product"),
        }
    }
}

pub fn render(meta: Head, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                title { (meta.title) }
                meta name="title" content=(meta.title);
                meta name="description" content=(meta.description);
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                meta property="og:type" content=(meta.og_type);
                meta property="og:url" content=(meta.url);
                meta property="og:title" content=(meta.title);
                meta property="og:description" content=(meta.description);
                link rel="alternate" type="application/rss+xml" title="Arne's Writing" href="/writing/atom.xml";
                link rel="alternate" type="application/rss+xml" title="Arne's Weekly" href="/weekly/atom.xml";
                link rel="alternate" type="application/rss+xml" title="Arne's Reading" href="/reading/atom.xml";
                link rel="me" href="https://spezi.social/@arne";
            }
            body {
                a.skip-link href="#main" { "Skip to content" }
                .sitewrapper {
                    nav.nav {
                        h1.nav__title { a href="/" { "Arne Bahlo" } }
                        a href="/articles" { "Articles" }
                        span.nav__separator { " | " }
                        a href="/weekly" { "Weekly" }
                        span.nav__separator { " | " }
                        a href="/projects" { "Projects" }
                        span.nav__separator { " | " }
                        a href="/contact" { "Contact" }
                    }
                    (content)
                    footer.footer {
                        span { (PreEscaped("&copy; 2021 &ndash; ")) (Utc::now().format("%Y")) " Arne Bahlo" }
                    }
                }
            }
        }
    }
}
