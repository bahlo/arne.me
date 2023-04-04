import { getCollection, getEntryBySlug } from "astro:content";
import renderOgImage from "../../../shared/renderOgImage.ts";

export async function getStaticPaths() {
  const posts = await getCollection("blog");
  return posts.map((post) => ({
    params: { slug: post.slug },
    props: post,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  const post = await getEntryBySlug("blog", params.slug);
  return renderOgImage(post.data.title, post.data.description);
};
