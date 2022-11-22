import { notFound } from "next/navigation";
import { renderMarkdown } from "../../../lib/markdown";
import { buildAbsolutePath } from "../../../lib/fs";
import { promises as fs } from "node:fs";
import { getPost } from "../../../lib/posts";

export async function generateStaticParams() {
  const path = buildAbsolutePath("content/blog");
  const files = await fs.readdir(path);
  return files
    .filter((filename) => filename != "_index.md" && filename.endsWith(".md"))
    .map((filename) => ({
      slug: filename.substring(0, filename.length - 3),
    }));
}

export default async function Blogpost({
  params: { slug },
}: {
  params: { slug: string };
}) {
  const post = await getPost(slug);
  if (!post) {
    return notFound();
  }

  let coverImage = null;
  if (post.frontmatter.coverImage) {
    const captionHtml = await renderMarkdown(
      post.frontmatter.coverImage.caption
    );
    coverImage = (
      <figure className="cover-image has-caption">
        <img
          src={post.frontmatter.coverImage.src}
          alt={post.frontmatter.coverImage.alt}
        />
        <figcaption dangerouslySetInnerHTML={{ __html: captionHtml }} />
      </figure>
    );
  }

  return (
    <article>
      <h1>{post.frontmatter.title}</h1>
      <span className="details">
        <time dateTime={post.frontmatter.date.toISOTime()}>
          {post.frontmatter.date.toFormat("LLL dd, yyyy")}
        </time>{" "}
        &middot; {post.readingTimeMinutes} min
      </span>
      {coverImage}
      <div dangerouslySetInnerHTML={{ __html: post.contentHtml! }} />
    </article>
  );
}
