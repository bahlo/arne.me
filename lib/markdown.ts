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

export async function parseMarkdown(
  ...segments: string[]
): Promise<Markdown | undefined> {
  const path = buildAbsolutePath(...segments);

  // Return undefined if the file doesn't exist
  let source;
  try {
    source = await fs.readFile(path, "utf-8");
  } catch {
    return undefined;
  }

  const { data: frontmatter, content } = matter(source);
  const html = await renderMarkdown(content);
  return {
    frontmatter,
    html,
  };
}

export async function parseFrontmatter(
  ...segments: string[]
): Promise<{ [key: string]: any } | undefined> {
  const path = buildAbsolutePath(...segments);

  let source;
  try {
    source = await fs.readFile(path, "utf-8");
  } catch {
    return undefined;
  }

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
