import { parseMarkdown } from "../../lib/markdown";
import SubscribeForm from "./SubscribeForm";
import { getIssues } from "../../lib/issues";

export default async function Weekly() {
  const page = await parseMarkdown("content/weekly/_index.md");
  const { frontmatter, html } = page!;
  const issues = await getIssues({ renderContent: false });

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />

      <SubscribeForm />

      <h2>Archive</h2>
      <ul>
        {issues.map(({ num, frontmatter }) => (
          <li key={num}>
            <a href={"/weekly/" + num}>{frontmatter.title}</a>
          </li>
        ))}
      </ul>
    </section>
  );
}
