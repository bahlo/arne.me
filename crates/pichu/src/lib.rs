use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(feature = "markdown")]
use comrak::{markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter};
#[cfg(feature = "markdown")]
use gray_matter::{engine::YAML, Matter};
#[cfg(feature = "markdown")]
use serde::de::DeserializeOwned;
#[cfg(feature = "markdown")]
use std::{fmt, fs::File, io::Read};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("{0}")]
    GlobError(#[from] glob::GlobError),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "markdown")]
    #[error("missing frontmatter in {0}")]
    MissingFrontmatter(PathBuf),
    #[cfg(feature = "markdown")]
    #[error("failed to deserialize frontmatter for {0}: {1}")]
    DeserializeFrontmatter(PathBuf, serde_json::error::Error),
    #[cfg(feature = "markdown")]
    #[error("no file stem for: {0}")]
    NoFileStem(PathBuf),
    #[error("render fn error: {0}")]
    RenderFn(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(feature = "sass")]
    #[error("failed to compile sass: {0}")]
    SassCompile(#[from] Box<grass::Error>),
}

pub fn glob(glob: impl AsRef<str>) -> Result<Glob, Error> {
    let paths = glob::glob(glob.as_ref())?;
    Ok(Glob { paths })
}

pub fn write(contents: impl AsRef<str>, to: impl AsRef<Path>) -> Result<(), Error> {
    // Create directory tree
    if let Some(parent) = to.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(to.as_ref(), contents.as_ref())?;
    Ok(())
}

#[derive(Debug)]
pub struct Glob {
    paths: glob::Paths,
}

impl Glob {
    pub fn parse<T>(
        self,
        parse_fn: impl Fn(PathBuf) -> Result<T, Error>,
    ) -> Result<Parsed<T>, Error> {
        let inner = self
            .paths
            .map(|path| parse_fn(path?))
            .collect::<Result<Vec<T>, Error>>()?;
        Ok(Parsed { items: inner })
    }

    #[cfg(feature = "markdown")]
    pub fn parse_markdown<T: DeserializeOwned + fmt::Debug>(
        self,
    ) -> Result<Parsed<Markdown<T>>, Error> {
        let syntect_adapter = SyntectAdapter::new(None);
        let markdown_context = MarkdownContext::new(&syntect_adapter);
        let matter = Matter::<YAML>::new();

        let mut parsed = vec![];
        for path in self.paths {
            let path = path?;
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let markdown = matter.parse(&contents);
            let frontmatter: T = markdown
                .data
                .ok_or(Error::MissingFrontmatter(path.clone()))?
                .deserialize()
                .map_err(|e| Error::DeserializeFrontmatter(path.clone(), e))?;

            let html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            let basename = path
                .file_stem()
                .ok_or_else(|| Error::NoFileStem(path.clone()))?
                .to_string_lossy()
                .to_string();

            parsed.push(Markdown {
                frontmatter,
                basename,
                html,
            })
        }

        Ok(Parsed { items: parsed })
    }
}

#[cfg(feature = "markdown")]
#[derive(Debug, Clone)]
pub struct Markdown<T> {
    pub frontmatter: T,
    pub basename: String,
    pub html: String,
}

#[derive(Debug, Clone)]
pub struct Parsed<T> {
    pub items: Vec<T>,
}

impl<T> Parsed<T> {
    pub fn sort_by_key<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self
    }

    pub fn sort_by_key_reverse<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self.items.reverse();
        self
    }

    pub fn render_each<
        P: AsRef<Path>,
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        self,
        render_fn: impl Fn(&T) -> Result<S, E>,
        build_path_fn: impl Fn(&T) -> P,
    ) -> Result<Self, Error> {
        // Render templates
        let rendered = self
            .items
            .iter()
            .map(|item| Ok((item, render_fn(item)?)))
            .collect::<Result<Vec<_>, E>>()
            .map_err(|e| Error::RenderFn(e.into()))?;

        // Write to disk
        rendered
            .into_iter()
            .map(|(item, content)| write(content.into(), build_path_fn(item)))
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(self)
    }

    pub fn render_all<S: Into<String>, E: Into<Box<dyn std::error::Error + Send + Sync>>>(
        self,
        render_fn: impl Fn(&Vec<T>) -> Result<S, E>,
        dest_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let content = render_fn(&self.items).map_err(|e| Error::RenderFn(e.into()))?;
        write(content.into(), dest_path)?;
        Ok(self)
    }
}

#[cfg(feature = "markdown")]
struct MarkdownContext<'a> {
    plugins: comrak::Plugins<'a>,
    options: comrak::Options<'a>,
}

#[cfg(feature = "markdown")]
impl<'a> MarkdownContext<'a> {
    fn new(syntect_adapter: &'a SyntectAdapter) -> Self {
        let mut render = comrak::RenderOptions::default();
        render.unsafe_ = true;
        let mut extension = comrak::ExtensionOptions::default();
        extension.strikethrough = true;
        extension.tagfilter = true;
        extension.table = true;
        extension.superscript = true;
        extension.header_ids = Some("".to_string());
        extension.footnotes = true;
        extension.description_lists = true;
        let mut parse = comrak::ParseOptions::default();
        parse.smart = true;
        let options = comrak::Options {
            render,
            extension,
            parse,
        };
        let mut render_plugins = comrak::RenderPlugins::default();
        render_plugins.codefence_syntax_highlighter = Some(syntect_adapter);
        let mut plugins = comrak::Plugins::default();
        plugins.render = render_plugins;

        Self { plugins, options }
    }
}

#[cfg(feature = "sass")]
pub fn render_sass(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<String, Error> {
    let source = source.as_ref();
    let options = match source.parent() {
        Some(parent) => grass::Options::default().load_path(parent),
        None => grass::Options::default(),
    };
    let css = grass::from_path(source, &options)?;
    let hash: String = blake3::hash(css.as_bytes())
        .to_string()
        .chars()
        .take(16)
        .collect();
    write(css, destination)?;
    Ok(hash)
}
