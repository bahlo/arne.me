import rss from "@astrojs/rss";
import { getCollection } from "astro:content";
import { BLOG_TITLE, BLOG_DESCRIPTION } from "../../consts";
import sanitizeHtml from "sanitize-html";
import MarkdownIt from "markdown-it";

const parser = new MarkdownIt();

export async function get(context) {
  const posts = (await getCollection("blog"))
    .filter((post) => post.slug != "hello-world")
    .sort((a, b) => b.data.pubDate.valueOf() - a.data.pubDate.valueOf());

  return rss({
    title: BLOG_TITLE,
    description: BLOG_DESCRIPTION,
    site: context.site,
    items: posts.map((post) => ({
      ...post.data,
      link: `/blog/${post.slug}/`,
      content: sanitizeHtml(parser.render(post.body)),
    })),
    customData: "<language>en-us</language><link>https://arne.me/blog</link>",
  });
}
