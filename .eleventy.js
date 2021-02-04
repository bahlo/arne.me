const yaml = require("js-yaml");
const htmlmin = require("html-minifier");
const { DateTime } = require('luxon');
const pluginSyntaxHighlight = require("@11ty/eleventy-plugin-syntaxhighlight");
const pluginRss = require('@11ty/eleventy-plugin-rss');

const addCollection = function(eleventyConfig, name, glob) {
  if (!glob) {
    glob = name;
  }

  const now = new Date();
  eleventyConfig.addCollection(name, collection => {
    return collection
	.getFilteredByGlob(`./${glob}/*.md`)
	.filter(item => item.date <= now && !item.data.draft);
  });
}

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

  eleventyConfig.addFilter('debug', console.log);

  eleventyConfig.addPassthroughCopy('./*.{png,ico,webmanifest}');
  eleventyConfig.addPassthroughCopy('./posts/**/*.{png,jpg}');
  eleventyConfig.addPassthroughCopy('./admin/config.yml');
  eleventyConfig.addPassthroughCopy('./static/img');

  addCollection(eleventyConfig, "posts");
  addCollection(eleventyConfig, "links");
  addCollection(eleventyConfig, "books");
  addCollection(eleventyConfig, "photos");
  addCollection(eleventyConfig, "entries", '{posts,links,books,photos}');

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
