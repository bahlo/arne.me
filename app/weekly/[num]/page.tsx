import SubscribeForm from "../SubscribeForm";
import { getIssues, getIssue } from "../../../lib/issues";
import { notFound } from "next/navigation";

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
  const issue = await getIssue(num);
  if (!issue) {
    return notFound();
  }

  return (
    <>
      <span className="preface">
        This is issue #{num} of <a href="/weekly">Arnes Weekly</a>.
      </span>
      <article className="article--weekly">
        <h1>{issue.frontmatter.title}</h1>
        <span className="details">
          <time dateTime={issue.frontmatter.date.toISOTime()}>
            {issue.frontmatter.date.toFormat("LLL dd, yyyy")}
          </time>
        </span>
        <div dangerouslySetInnerHTML={{ __html: issue.contentHtml! }} />
        <br />
        <SubscribeForm />
      </article>
    </>
  );
}
