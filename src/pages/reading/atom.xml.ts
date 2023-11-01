import rss from "@astrojs/rss";
import { getCollection } from "astro:content";
import { READING_TITLE, READING_DESCRIPTION } from "../../consts";
import sanitizeHtml from "sanitize-html";
import MarkdownIt from "markdown-it";

const parser = new MarkdownIt();

export async function get(context) {
  const books = (await getCollection("reading")).sort(
    (a, b) => b.data.dateRead.valueOf() - a.data.dateRead.valueOf()
  );

  return rss({
    title: READING_TITLE,
    description: READING_DESCRIPTION,
    site: context.site,
    items: books.map((book) => ({
      title: book.data.title,
      pubDate: book.data.dateRead,
      description: book.data.subtitle,
      link: `/reading/${book.slug}/`,
      content: sanitizeHtml(parser.render(book.body)),
    })),
    customData:
      "<language>en-us</language><link>https://2023.arne.me/reading</link>",
  });
}
