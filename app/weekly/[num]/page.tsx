import SubscribeForm from "../SubscribeForm";
import { getIssues, getIssue } from "../../../lib/issues";

export async function generateStaticParams() {
  const issues = await getIssues({ renderContent: false });
  return issues.map((issue) => ({
    num: issue.num.toString(),
  }));
}

export default async function Issue({
  params: { num: numStr },
}: {
  params: { num: string };
}) {
  const num = parseInt(numStr, 10);
  const { frontmatter, contentHtml } = await getIssue(num);

  return (
    <>
      <span className="preface">
        This is issue #{num} of <a href="/weekly">Arnes Weekly</a>.
      </span>
      <article className="article--weekly">
        <h1>{frontmatter.title}</h1>
        <span className="details">
          <time dateTime={frontmatter.date.toISOTime()}>
            {frontmatter.date.toFormat("LLL dd, yyyy")}
          </time>
        </span>
        <div dangerouslySetInnerHTML={{ __html: contentHtml! }} />
        <br />
        <SubscribeForm />
      </article>
    </>
  );
}
