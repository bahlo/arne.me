import { parseMarkdown, buildAbsolutePath } from "../../lib/markdown";
import matter from "gray-matter";
import { promises as fs } from "node:fs";

async function parseIssues() {
  const path = buildAbsolutePath("content/weekly");
  const files = await fs.readdir(path);
  const parsed = await Promise.all(
    files
      .filter(
        (filename) =>
          filename != "_index.md" &&
          filename != "unsubscribed.md" &&
          filename != "subscribed.md" &&
          filename.endsWith(".md")
      )
      .map(async (filename) => {
        const slug = filename.substring(0, filename.length - 3);
        const source = await fs.readFile(
          buildAbsolutePath("content/weekly/" + filename)
        );
        const { data: frontmatter } = matter(source);
        return {
          slug,
          frontmatter,
        };
      })
  );
  return parsed.sort((a, b) => {
    const dateA = Date.parse(a.frontmatter.date);
    const dateB = Date.parse(b.frontmatter.date);
    return dateB - dateA;
  });
}

export default async function Weekly() {
  const { frontmatter, html } = await parseMarkdown("content/weekly/_index.md");
  const issues = await parseIssues();

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />

      <ul>
        {issues.map(({ frontmatter }) => (
          <li key={frontmatter.num}>
            <a href={"/weekly/" + frontmatter.num}>{frontmatter.title}</a>
          </li>
        ))}
      </ul>
    </section>
  );
}
