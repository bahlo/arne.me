import { parseMarkdown } from "../../lib/markdown";

export default async function Now() {
  const { frontmatter, html } = await parseMarkdown('content/now.md');

  return (
    <section>
      <h1>{frontmatter.hero}</h1>
      <div dangerouslySetInnerHTML={{__html: html.toString()}}/>
    </section>
  );
}