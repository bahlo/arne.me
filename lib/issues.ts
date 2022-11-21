import { promises as fs } from "fs";
import { buildAbsolutePath } from "./fs";
import matter from "gray-matter";
import { renderMarkdown } from "./markdown";
import { DateTime } from "luxon";

export interface Issue {
  num: number;
  frontmatter: {
    title: string;
    date: DateTime;
  };
  content: string;
  contentHtml?: string;
}

interface Opts {
  renderContent: boolean;
}

export async function getIssues(
  opts: Opts = { renderContent: false }
): Promise<Issue[]> {
  const path = buildAbsolutePath("content/weekly");
  const files = await fs.readdir(path);
  const issues = await Promise.all(
    files
      .filter(
        (filename) =>
          filename != "_index.md" &&
          filename != "subscribed.md" &&
          filename != "unsubscribed.md" &&
          filename.endsWith(".md")
      )
      .map(async (filename) => {
        const base = filename.substring(0, filename.length - 3);
        const num = parseInt(base, 10);
        const issue = await getIssue(num, opts);
        return issue;
      })
  );
  return issues.sort((a: Issue, b: Issue) => b.num - a.num);
}

export async function getIssue(
  num: number,
  opts: Opts = {
    renderContent: true,
  }
): Promise<Issue> {
  const path = buildAbsolutePath("content", "weekly", num + ".md");
  const source = await fs.readFile(path);
  const { data: frontmatter, content } = matter(source);
  let contentHtml;
  if (opts.renderContent) {
    contentHtml = await renderMarkdown(content);
  }

  return {
    num,
    frontmatter: {
      ...frontmatter,
      title: frontmatter.title!,
      date: DateTime.fromISO(frontmatter.date!),
    },
    content,
    contentHtml: contentHtml?.toString(),
  };
}
