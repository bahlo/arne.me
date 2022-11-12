import { promises as fs } from 'node:fs';
import matter from 'gray-matter';
import { remark } from 'remark';
import remarkHtml from 'remark-html';
import { join } from 'node:path';

interface Markdown {
  frontmatter: { [key: string]: any },
  html: string
}

export function buildAbsolutePath(...segments: string[]): string {
  const cwd = process.cwd();
  const root = cwd.replace(/\.next\/server\/app(\/\[[a-z.]+\])?(\/rsc)?/, '')
  return join(root, ...segments)
}

export async function parseMarkdown(...segments: string[]): Promise<Markdown> {
  const path = buildAbsolutePath(...segments);
  const source = await fs.readFile(path, 'utf-8');
  const { data: frontmatter, content } = matter(source);
  const html = await remark().use(remarkHtml).process(content);
  return {
    frontmatter, 
    html: html.toString()
  }
}