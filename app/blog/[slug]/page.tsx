import { buildAbsolutePath, parseMarkdown, renderMarkdown } from "../../../lib/markdown";
import { promises as fs } from 'node:fs';

export async function generateStaticParams() {
  const path = buildAbsolutePath('content/blog');
  const files = await fs.readdir(path);
  return files.filter(filename => filename.endsWith('.md')).map(filename => ({
    slug: filename.substring(0, filename.length-3),
  }));
}

export default async function Blogpost({params: { slug }}: { params: { slug: string }}) {
  const { frontmatter, html } = await parseMarkdown('content/blog/'+slug+'.md');

  const readingTime = "TODO"

  let coverImage = null;
  if (frontmatter.coverImage) {
    const captionHtml = await renderMarkdown(frontmatter.coverImage.caption);
    coverImage = (
      <figure className="cover-image has-caption">
        <img src={ frontmatter.coverImage.src } alt={ frontmatter.coverImage.alt }/>
        <figcaption dangerouslySetInnerHTML={{__html: captionHtml}} />
      </figure>
    )
  }

  return (
    <article>
      <h1>{frontmatter.title}</h1>
      <span className="details">
        <time dateTime="{ frontmatter.date }">{ frontmatter.date }</time> &middot; { readingTime } min
      </span>
      {coverImage}
      <div dangerouslySetInnerHTML={{__html: html.toString()}}/>
    </article>
  );
}