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

#[derive(Debug, Default)]
pub struct Options {
    pub is_index: bool,
    pub full_width: bool,
}

pub fn render(meta: Head, content: Markup, options: impl Into<Option<Options>>) -> Markup {
    let options = options.into().unwrap_or_default();
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                title { (meta.title) }
                meta name="title" content=(meta.title);
                meta name="description" content=(meta.description);
                meta name="author" content="Arne Bahlo";
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                meta property="og:type" content=(meta.og_type);
                meta property="og:url" content=(meta.url);
                meta property="og:title" content=(meta.title);
                meta property="og:description" content=(meta.description);
                link rel="sitemap" href="/sitemap.xml";
                link rel="stylesheet" href="/main.css";
                link rel="alternate" type="application/rss+xml" title="Arne's Writing" href="/writing/atom.xml";
                link rel="alternate" type="application/rss+xml" title="Arne's Weekly" href="/weekly/atom.xml";
                link rel="alternate" type="application/rss+xml" title="Arne's Reading" href="/reading/atom.xml";
                link rel="me" href="https://spezi.social/@arne";
            }
            body {
                a.skip-link href="#main" { "Skip to content" }
                div.sitewrapper.sitewrapper--page[!options.full_width] {
                    @if options.is_index {
                        .hero {
                            h1.hero__heading { "Hey, I'm Arne—" }
                            p {
                                "a developer, podcaster & dad based near Frankfurt, Germany. Having worked with a variety of technologies and stacks, from frontend to backend to databases, I know my way around building accessible and safe applications and tools. "
                                "In my free time I like to play with my kids, work on side projects, play chess and read books."
                            }
                        }
                    } @else {
                        a href="/" { "← Index" }
                    }
                    // nav.nav {
                    //     a.nav__title href="/" { "Arne Bahlo" }
                    //     span.nav__separator { " // " }
                    //     a.nav__link href="/projects" { "Projects" }
                    //     span.nav__separator { " // " }
                    //     a.nav__link href="/contact" { "Contact" }
                    // }
                    main #main {
                        (content)
                    }
                    br;
                    footer.footer {
                        span.footer_copyright {
                            (PreEscaped("&copy; 2021 &ndash; ")) (Utc::now().format("%Y")) " Arne Bahlo"
                        }
                        br;
                        span.footer__pages {
                            a href="/projects" { "Projects" }
                            " // "
                            a href="/contact" { "Contact" }
                            " // "
                            a href="/imprint" { "Imprint" }
                        }
                    }
                }
            }
        }
    }
}
