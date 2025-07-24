use anyhow::{anyhow, Context, Result};
use core::str;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};
use tiny_skia::Pixmap;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;
const CHARS_PER_LINE: usize = 22;

pub fn generate(title: impl Into<String>, output_file: impl AsRef<Path>) -> Result<()> {
    println!("Generating {:?}", output_file.as_ref());
    let title = title.into();
    let tspans = title
        .split(' ')
        .fold(vec![], |mut lines, word| {
            match lines.last_mut() {
                None => {
                    // Create first line
                    lines.push(word.to_string());
                }
                Some(line) if line.len() + word.len() >= CHARS_PER_LINE => {
                    // We need a new line
                    lines.push(word.to_string());
                }
                Some(line) => {
                    *line = format!("{line} {word}");
                }
            }
            lines
        })
        .iter()
        .enumerate()
        .map(|(pos, line)| {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_truncation)]
            let dy = (pos as f32 * 96.0 * 1.2) as i32; // 96px font-size, 1.2 line-height
            let line = html_escape::encode_text(line);
            format!(r#"<tspan x="64" y="152.1" dy="{dy}">{line}</tspan>"#)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let svg = include_bytes!("../static/og-template.svg");
    let svg = str::from_utf8(svg)?.replace("{{ tspans }}", &tspans);

    let mut font_data = vec![];
    File::open("static/fonts/rebond-grotesque/ESRebondGrotesque-Bold.ttf")
        .context("Failed to open font")?
        .read_to_end(&mut font_data)?;
    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).ok_or(anyhow!("Pixmap allocation error"))?;
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_font_data(font_data);
    let tree = usvg::Tree::from_str(&svg, &options)?;
    resvg::render(&tree, usvg::Transform::identity(), &mut pixmap.as_mut());
    let png_data = pixmap.encode_png()?;
    fs::write(output_file.as_ref(), png_data)?;

    Ok(())
}
