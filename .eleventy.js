const htmlmin = require("html-minifier");
const pluginRss = require('@11ty/eleventy-plugin-rss');

module.exports = function (eleventyConfig) {
  eleventyConfig.addPlugin(pluginRss);
  eleventyConfig.setUseGitIgnore(false);
 
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

  // Favicons
  eleventyConfig.addPassthroughCopy('*.{png,ico,webmanifest}');

  // Post images
  eleventyConfig.addPassthroughCopy('posts/**/*.{png,jpg}');

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
