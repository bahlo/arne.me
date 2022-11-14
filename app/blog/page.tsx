import { parseMarkdown, buildAbsolutePath, renderMarkdown } from "../../lib/markdown";
import matter from 'gray-matter';
import {promises as fs} from 'node:fs';

async function parseBlogposts() {
  const path = buildAbsolutePath('content/blog');
  const files = await fs.readdir(path);
  return Promise.all(files.filter(filename => filename != "_index.md" && filename.endsWith('.md')).map(async filename => {
    const slug = filename.substring(0, filename.length - 3);
    const source = await fs.readFile(buildAbsolutePath('content/blog/'+filename))
    const { data: frontmatter } = matter(source);
    const descriptionHtml = await renderMarkdown(frontmatter.description);
    return {
      slug,
      frontmatter,
      descriptionHtml,
    }
  }))
}

export default async function Blog() {
  const { frontmatter, html } = await parseMarkdown('content/blog/_index.md');
  const blogposts = await parseBlogposts();

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <div dangerouslySetInnerHTML={{__html: html.toString()}}/>
      {blogposts.map(({ frontmatter, slug, descriptionHtml }, i) => (
        <article className="article--list" key={slug}>
          <h1><a href={"/blog/" + slug }>{ frontmatter.title }</a></h1>
          <span className="details">
            <time dateTime="{ frontmatter.date}">{ frontmatter.date }</time> &middot; { frontmatter.reading_time } min
          </span>
          <div dangerouslySetInnerHTML={{__html: descriptionHtml}}/>
        </article>
      ))}
    </section>
  );
}