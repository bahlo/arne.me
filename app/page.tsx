import { parseMarkdown } from "../lib/markdown";

export default async function Page() {
  const { frontmatter, html } = await parseMarkdown('content/index.md');

  return (<>
    <div>
      <h1>{frontmatter.hero}</h1>
      <div dangerouslySetInnerHTML={{__html: html.toString()}}/>
    </div>
  </>);
}