import rss from "@astrojs/rss";
import { getCollection } from "astro:content";
import { WEEKLY_TITLE, WEEKLY_DESCRIPTION } from "../../consts.ts";
import renderWeeklyIssue from "../../shared/WeeklyIssueContent.ts";
import sanitizeHtml from "sanitize-html";
import MarkdownIt from "markdown-it";

const parser = new MarkdownIt();

export async function get(context) {
  const issues = (await getCollection("weekly")).sort(
    (a, b) => b.data.date.valueOf() - a.data.date.valueOf()
  );

  return rss({
    title: WEEKLY_TITLE,
    description: WEEKLY_DESCRIPTION,
    site: context.site,
    items: issues.map((issue) => ({
      title: issue.data.title,
      pubDate: issue.data.date,
      description: `Issue #${issue.slug} from Arne's Weekly`,
      link: `/weekly/${issue.slug}`,
      content: sanitizeHtml(
        parser.render(issue.body) + renderWeeklyIssue(issue.slug, issue.data)
      ),
    })),
    customData: "<language>en-us</language><link>https://2023.arne.me/weekly</link>",
  });
}
