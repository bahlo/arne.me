import { parseMarkdown } from "../lib/markdown";

export default async function Page() {
  const page = await parseMarkdown("content/index.md");
  const { frontmatter, html } = page!;

  return (
    <section className="intro">
      <h1 className="title">{frontmatter.hero}</h1>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />
    </section>
  );
}
