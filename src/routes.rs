use maud::{html, Markup};
use url::Url;

use crate::layout::{self, Head, OgType};

pub async fn index() -> Markup {
    layout::render(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me").unwrap(),
            og_type: OgType::Website,
        },
        html! { h1 { "Hello, world!" } },
    )
}
