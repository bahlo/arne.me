import { promises as fs } from "node:fs";
import RSS from "rss";
import { getPosts, Post } from "../lib/posts.js";
import { getIssues, Issue } from "../lib/issues.js";

async function generateBlogFeed() {
  const blogposts = await getPosts({
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

  blogposts
    .sort(
      (a: Post, b: Post) =>
        b.frontmatter.date.toUnixInteger() - a.frontmatter.date.toUnixInteger()
    )
    .forEach((post: Post) => {
      feed.item({
        title: post.frontmatter.title,
        guid: `https://arne.me/blog/${post.slug}`,
        url: `https://arne.me/blog/${post.slug}`,
        date: post.frontmatter.date.toJSDate(),
        description: post.contentHtml!,
        author: "Arne Bahlo",
      });
    });

  const rss = feed.xml();
  await fs.writeFile("./public/blog/atom.xml", rss);
}

async function generateWeeklyFeed() {
  const issues = await getIssues({ renderContent: true });

  const feed = new RSS({
    title: "Arne's Weekly",
    site_url: "https://arne.me",
    feed_url: "https://arne.me/weekly/atom.xml",
    language: "en",
  });

  issues
    .sort(
      (a: Issue, b: Issue) =>
        b.frontmatter.date.toUnixInteger() - a.frontmatter.date.toUnixInteger()
    )
    .forEach((issue: Issue) => {
      feed.item({
        title: issue.frontmatter.title,
        guid: `https://arne.me/weekly/${issue.num}`,
        url: `https://arne.me/weekly/${issue.num}`,
        date: issue.frontmatter.date.toJSDate(),
        description: issue.contentHtml!,
        author: "Arne Bahlo",
      });
    });

  const rss = feed.xml();
  await fs.writeFile("./public/weekly/atom.xml", rss);
}

async function main() {
  await generateBlogFeed();
  await generateWeeklyFeed();
}

main();
