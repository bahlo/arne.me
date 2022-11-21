import { buildAbsolutePath } from "../../lib/fs";
import { parseMarkdown } from "../../lib/markdown";
import { promises as fs } from "node:fs";

export async function generateStaticParams() {
  const path = buildAbsolutePath("content");
  const files = await fs.readdir(path);
  return files
    .filter((filename) => filename.endsWith(".md") && filename != "index.md")
    .map((filename) => ({
      page: filename.substring(0, filename.length - 3),
    }));
}

export default async function Page({
  params: { page },
}: {
  params: { page: string };
}) {
  const { frontmatter, html } = await parseMarkdown("content/" + page + ".md");

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />
    </section>
  );
}
