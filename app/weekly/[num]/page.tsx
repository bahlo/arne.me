import { buildAbsolutePath, parseMarkdown } from "../../../lib/markdown";
import { promises as fs } from "node:fs";

export async function generateStaticParams() {
  const path = buildAbsolutePath("content/weekly");
  const files = await fs.readdir(path);
  return files
    .filter((filename) => filename != "_index.md" && filename.endsWith(".md"))
    .map((filename) => ({
      num: filename.substring(0, filename.length - 3),
    }));
}

export default async function Issue({
  params: { num },
}: {
  params: { num: string };
}) {
  const { frontmatter, html } = await parseMarkdown(
    "content/weekly/" + num + ".md"
  );

  return (
    <>
      <span className="preface">
        This is issue #{frontmatter.num} of <a href="/weekly">Arnes Weekly</a>.
      </span>
      <article className="article--weekly">
        <h1>{frontmatter.title}</h1>
        <span className="details">
          <time dateTime="{ frontmatter.date}">{frontmatter.date}</time>
        </span>
        <div dangerouslySetInnerHTML={{ __html: html }} />
      </article>
    </>
  );
}
