import { promises as fs } from 'node:fs';
import matter from 'gray-matter';
import { remark } from 'remark';
import remarkHtml from 'remark-html';
import { join } from 'node:path';

interface Markdown {
  frontmatter: { [key: string]: any },
  html: string
}

function buildPath(...segments: string[]): string {
  const cwd = process.cwd();
  if (cwd.endsWith('/.next/server/app')) {
    // Turbopack has a different cwd
    return join(cwd, '..', '..', '..', ...segments)
  } else {
    return join(cwd, ...segments)
  }
}

export async function parseMarkdown(...segments: string[]): Promise<Markdown> {
  const path = buildPath(...segments);
  const source = await fs.readFile(path, 'utf-8');
  const { data: frontmatter, content } = matter(source);
  const html = await remark().use(remarkHtml).process(content);
  return {
    frontmatter, 
    html: html.toString()
  }
}