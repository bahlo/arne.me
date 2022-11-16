import { promises as fs } from "node:fs";
import matter from "gray-matter";
import { remark } from "remark";
import remarkHtml from "remark-html";
import { join } from "node:path";
import calculateReadingTimeMinutes from "./readingTime";

interface Markdown {
  frontmatter: { [key: string]: any };
  readingTimeMinutes: number;
  html: string;
}

export function buildAbsolutePath(...segments: string[]): string {
  const cwd = process.cwd();
  const root = cwd.replace(/\.next\/server\/app(\/[a-z.\[\]]+)*(\/rsc)?/, "");
  return join(root, ...segments);
}

export async function parseMarkdown(...segments: string[]): Promise<Markdown> {
  const path = buildAbsolutePath(...segments);
  const source = await fs.readFile(path, "utf-8");
  const { data: frontmatter, content } = matter(source);
  const html = await renderMarkdown(content);
  const readingTimeMinutes = calculateReadingTimeMinutes(content);
  return {
    frontmatter,
    readingTimeMinutes,
    html,
  };
}

export async function parseFrontmatter(
  ...segments: string[]
): Promise<{ [key: string]: any }> {
  const path = buildAbsolutePath(...segments);
  const source = await fs.readFile(path, "utf-8");
  const { data: frontmatter } = matter(source);
  return frontmatter;
}

export async function renderMarkdown(source: string): Promise<string> {
  const html = await remark().use(remarkHtml).process(source);
  return html.toString();
}
