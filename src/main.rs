use serde::Serialize;
use std::process::exit;
use tera::Tera;
use url::Url;

#[derive(Serialize)]
struct Context {
    title: String,
    description: String,
    url: Url,
}

impl From<Context> for tera::Context {
    fn from(context: Context) -> Self {
        let mut tera_context = tera::Context::new();
        tera_context.insert("title", &context.title);
        tera_context.insert("description", &context.description);
        tera_context.insert("url", &context.url);
        tera_context
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            exit(1);
        }
    };

    let context = Context {
        title: "Arne Bahlo".to_string(),
        description: "Arne Bahlo's personal website".to_string(),
        url: Url::parse("https://arne.me")?,
    };
    let output = tera.render("layout.html", &context.into())?;

    println!("{}", output);

    Ok(())
}
