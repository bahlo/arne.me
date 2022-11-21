import { promises as fs } from "node:fs";
import matter from "gray-matter";
import { remark } from "remark";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import rehypePrism from "@mapbox/rehype-prism";
import { buildAbsolutePath } from "./fs";
import rehypeStringify from "rehype-stringify";

interface Markdown {
  frontmatter: { [key: string]: any };
  html: string;
}

export async function parseMarkdown(...segments: string[]): Promise<Markdown> {
  const path = buildAbsolutePath(...segments);
  const source = await fs.readFile(path, "utf-8");
  const { data: frontmatter, content } = matter(source);
  const html = await renderMarkdown(content);
  return {
    frontmatter,
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
  const html = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypePrism)
    .use(rehypeStringify)
    .process(source);
  return html.toString();
}