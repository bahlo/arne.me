const yaml = require("js-yaml");
const htmlmin = require("html-minifier");
const { DateTime } = require('luxon');
const pluginSyntaxHighlight = require("@11ty/eleventy-plugin-syntaxhighlight");
const pluginRss = require('@11ty/eleventy-plugin-rss');

module.exports = function (eleventyConfig) {
  eleventyConfig.addPlugin(pluginRss);
  eleventyConfig.addPlugin(pluginSyntaxHighlight);

  eleventyConfig.setUseGitIgnore(false);

  eleventyConfig.addDataExtension("yaml", yaml.load);
 
  eleventyConfig.addShortcode("version", () => {
    return String(Date.now());
  });
 
  eleventyConfig.addShortcode("year", () => {
    let now = new Date();
    return String(now.getFullYear());
  });

  eleventyConfig.addFilter('readableDate', dateObj => {
    return DateTime.fromJSDate(dateObj, {zone: 'utc'}).toFormat('dd LLL yyyy');
  });

  // https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#valid-date-string
  eleventyConfig.addFilter('htmlDateString', dateObj => {
    return DateTime.fromJSDate(dateObj, {zone: 'utc'}).toFormat('yyyy-LL-dd');
  });

  eleventyConfig.addPassthroughCopy('./*.{png,ico,webmanifest}');
  eleventyConfig.addPassthroughCopy('./posts/**/*.{png,jpg}');
  eleventyConfig.addPassthroughCopy('./admin/config.yml');
  eleventyConfig.addPassthroughCopy('./static/img');

  const now = new Date();
  eleventyConfig.addCollection("posts", collection => {
    return collection
	.getFilteredByGlob("./posts/*.md")
	.filter(post => post.date <= now && !post.data.draft);
  });

  eleventyConfig.addCollection("links", collection => {
    return collection
	.getFilteredByGlob("./links/*.md")
	.filter(link => link.date <= now && !link.data.draft);
  });
  eleventyConfig.addCollection("books", collection => {
    return collection
	.getFilteredByGlob("./books/*.md")
	.filter(book => book.date <= now && !book.data.draft);
  });
  eleventyConfig.addCollection("photos", collection => {
    return collection
	.getFilteredByGlob("./photos/*.md")
	.filter(photo => photo.date <= now && !photo.data.draft);
  });
  eleventyConfig.addCollection("entries", collection => {
    return collection
	.getFilteredByGlob("./{posts,links,books,photos}/*.md")
	.filter(entry => entry.date <= now && !entry.data.draft);
  });

  eleventyConfig.addTransform("htmlmin", function (content, outputPath) {
    if (
      process.env.ELEVENTY_PRODUCTION &&
      outputPath &&
      outputPath.endsWith(".html")
    ) {
      let minified = htmlmin.minify(content, {
        useShortDoctype: true,
        removeComments: true,
        collapseWhitespace: true,
      });
      return minified;
    }
 
    return content;
  });
};
