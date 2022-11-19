import { promises as fs } from "node:fs";
import RSS from "rss";
import { getPosts, Post } from "../lib/posts.js";
// const RSS = require("rss");
// const { promises: fs } = require("node:fs");
// const { getPosts } = require("../lib/posts");
// import { Post } from "../lib/posts";

async function generateBlogFeed() {
  const previewItems = await getPosts({
    renderContent: true,
    renderDescription: false,
    renderCoverImageCaption: false,
  });

  const feed = new RSS({
    title: "Arne Bahlo — Blog",
    site_url: "https://arne.me",
    feed_url: "https://arne.me/blog/atom.xml",
    language: "en",
  });

  previewItems
    .sort(
      (a: Post, b: Post) =>
        b.frontmatter.date.toUnixInteger() - a.frontmatter.date.toUnixInteger()
    )
    .forEach((post: Post) => {
      feed.item({
        title: post.frontmatter.title,
        guid: post.slug,
        url: `https://arne.me/blog/${post.slug}`,
        date: post.frontmatter.date.toJSDate(),
        description: post.contentHtml!,
        author: "Arne Bahlo",
      });
    });

  const rss = feed.xml();
  await fs.writeFile("./public/blog/atom.xml", rss);
}

async function main() {
  await generateBlogFeed();
}

main();
