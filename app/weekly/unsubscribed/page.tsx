import { parseMarkdown } from "../../../lib/markdown";

export default async function Unsubscribed() {
  const { frontmatter, html } = await parseMarkdown(
    "content/weekly/unsubscribed.md"
  );

  return (
    <>
      <section>
        <h1>{frontmatter.title}</h1>
        <div dangerouslySetInnerHTML={{ __html: html }} />
      </section>
    </>
  );
}
