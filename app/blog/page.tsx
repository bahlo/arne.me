import { parseMarkdown } from "../../lib/markdown";
import { getPosts } from "../../lib/posts";

export default async function Blog() {
  const { frontmatter, html } = await parseMarkdown("content/blog/_index.md");
  const blogposts = await getPosts();

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />
      {blogposts
        .sort((a, b) => {
          return (
            b.frontmatter.date.toUnixInteger() -
            a.frontmatter.date.toUnixInteger()
          );
        })
        .map(({ frontmatter, slug, readingTimeMinutes }) => (
          <article className="article--list" key={slug}>
            <h1>
              <a href={"/blog/" + slug}>{frontmatter.title}</a>
            </h1>
            <span className="details">
              <time dateTime="{ frontmatter.date}">
                {frontmatter.date.toFormat("LLL dd, yyyy")}
              </time>{" "}
              &middot; {readingTimeMinutes} min
            </span>
            <div
              dangerouslySetInnerHTML={{ __html: frontmatter.descriptionHtml! }}
            />
          </article>
        ))}
    </section>
  );
}
