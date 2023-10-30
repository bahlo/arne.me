use maud::html;
use url::Url;

use crate::layout::{Meta, OgType};

mod layout;
mod routes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = Meta {
        title: "Arne Bahlo".to_string(),
        description: "Arne Bahlo's personal website".to_string(),
        url: Url::parse("https://arne.me")?,
        og_type: OgType::Website,
    };

    let markup = layout::render(meta, html! { h1 { "Hello, world!" } });

    println!("{}", markup.into_string());

    Ok(())
}
