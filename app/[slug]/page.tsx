import { notFound } from "next/navigation";
import { buildAbsolutePath } from "../../lib/fs";
import { parseMarkdown } from "../../lib/markdown";
import { promises as fs } from "node:fs";

export async function generateStaticParams() {
  const path = buildAbsolutePath("content");
  const files = await fs.readdir(path);
  return files
    .filter((filename) => filename.endsWith(".md") && filename != "index.md")
    .map((filename) => ({
      slug: filename.substring(0, filename.length - 3),
    }));
}

export default async function Page({
  params: { slug },
}: {
  params: { slug: string };
}) {
  const page = await parseMarkdown("content/" + slug + ".md");
  if (!page) {
    return notFound();
  }

  return (
    <section>
      <h1>{page.frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: page.html.toString() }} />
    </section>
  );
}
