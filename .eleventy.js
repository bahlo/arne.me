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

  eleventyConfig.addPassthroughCopy('*.{png,ico,webmanifest}');

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
