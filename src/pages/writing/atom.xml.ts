import rss from "@astrojs/rss";
import { getCollection } from "astro:content";
import { WRITING_TITLE, WRITING_DESCRIPTION } from "../../consts";
import sanitizeHtml from "sanitize-html";
import MarkdownIt from "markdown-it";

const parser = new MarkdownIt();

export async function get(context) {
  const posts = (await getCollection("writing"))
    .filter((post) => post.slug != "hello-world")
    .sort((a, b) => b.data.pubDate.valueOf() - a.data.pubDate.valueOf());

  return rss({
    title: WRITING_TITLE,
    description: WRITING_DESCRIPTION,
    site: context.site,
    items: posts.map((post) => ({
      ...post.data,
      link: `/writing/${post.slug}/`,
      content: sanitizeHtml(parser.render(post.body)),
    })),
    customData:
      "<language>en-us</language><link>https://arne.me/writing</link>",
  });
}
