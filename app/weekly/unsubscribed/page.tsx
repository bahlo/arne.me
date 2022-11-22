import { parseMarkdown } from "../../../lib/markdown";

export default async function Unsubscribed() {
  const page = await parseMarkdown("content/weekly/unsubscribed.md");
  const { frontmatter, html } = page!;

  return (
    <>
      <section>
        <h1>{frontmatter.title}</h1>
        <div dangerouslySetInnerHTML={{ __html: html }} />
      </section>
    </>
  );
}
