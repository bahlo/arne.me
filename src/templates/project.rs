use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::Project,
    layout::{self, Head, OgType},
};

fn render_project(project: &Project) -> Markup {
    html! {
        details.project open[project.to.is_none()] {
            summary {
                strong {
                    @if let Some(url) = &project.url {
                        a href=(url) {
                            (project.title)
                        }
                    } @else {
                        (project.title)
                    }
                }
                " (" (project.from) (PreEscaped(" &ndash; "))
                @if let Some(to) = &project.to {
                     (to)
                } @else {
                    "Present"
                }
                ")"
            }

            .project__description {
                (PreEscaped(project.content_html.clone()))
            }
        }
    }
}

pub fn render(project: &[Project], css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Projects",
            description: "Some projects I've worked on",
            url: Url::parse("https://arne.me/projects")?,
            og_type: OgType::Website,
            css_hash: css_hash.as_ref(),
        },
        html! {
            article.article {
                header {
                    h1 { "Projects" }
                }
                p { "Here are the projects I'm currently working on:" }
                @for project in project.iter().filter(|project| project.to.is_none()) {
                    (render_project(project))
                }

                h2 { "Inactive/Abandoned Projects" }
                @for project in project.iter().filter(|project| project.to.is_some()) {
                    (render_project(project))
                }
            }
        },
        None,
    ))
}
