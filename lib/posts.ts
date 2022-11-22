import { promises as fs } from "fs";
import { buildAbsolutePath } from "./fs";
import matter from "gray-matter";
import { renderMarkdown } from "./markdown";
import calculateReadingTimeMinutes from "./readingTime";
import { DateTime } from "luxon";

export interface Post {
  slug: string;
  frontmatter: {
    title: string;
    description: string;
    descriptionHtml?: string;
    date: DateTime;
    coverImage?: {
      src: string;
      alt: string;
      caption: string;
      captionHtml?: string;
    };
  };
  content: string;
  contentHtml?: string;
  readingTimeMinutes: number;
}

interface Opts {
  renderDescription: boolean;
  renderContent: boolean;
  renderCoverImageCaption: boolean;
}

export async function getPosts(
  opts: Opts = {
    renderDescription: true,
    renderContent: false,
    renderCoverImageCaption: false,
  }
): Promise<Post[]> {
  const path = buildAbsolutePath("content/blog");
  const files = await fs.readdir(path);
  return Promise.all(
    files
      .filter((filename) => filename != "_index.md" && filename.endsWith(".md"))
      .map(async (filename) => {
        const slug = filename.substring(0, filename.length - 3);
        const post = await getPost(slug, opts);
        return post!;
      })
  );
}

export async function getPost(
  slug: string,
  opts: Opts = {
    renderDescription: false,
    renderContent: true,
    renderCoverImageCaption: true,
  }
): Promise<Post | undefined> {
  const path = buildAbsolutePath("content", "blog", slug + ".md");

  let source;
  try {
    source = await fs.readFile(path);
  } catch {
    return undefined;
  }

  const { data: frontmatter, content } = matter(source);
  let descriptionHtml, contentHtml;
  if (opts.renderDescription) {
    descriptionHtml = await renderMarkdown(frontmatter.description);
  }
  if (opts.renderContent) {
    contentHtml = await renderMarkdown(content);
  }
  let coverImage;
  if (frontmatter.coverImage) {
    let captionHtml;
    if (opts.renderCoverImageCaption) {
      captionHtml = await renderMarkdown(frontmatter.coverImage.description);
    }
    coverImage = {
      src: frontmatter.coverImage.src!,
      alt: frontmatter.coverImage.alt!,
      caption: frontmatter.coverImage.caption!,
      captionHtml,
    };
  }
  const readingTimeMinutes = calculateReadingTimeMinutes(content);
  return {
    slug,
    frontmatter: {
      ...frontmatter,
      title: frontmatter.title!,
      description: frontmatter.description!,
      descriptionHtml: descriptionHtml?.toString(),
      date: DateTime.fromISO(frontmatter.date!),
      coverImage,
    },
    content,
    contentHtml: contentHtml?.toString(),
    readingTimeMinutes,
  };
}
