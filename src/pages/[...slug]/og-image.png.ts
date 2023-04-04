import { getCollection, getEntryBySlug } from "astro:content";
import renderOgImage from "../../shared/renderOgImage.ts";

export async function getStaticPaths() {
  const pages = await getCollection("pages");
  return pages.map((page) => ({
    params: { slug: page.slug },
    props: page,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  const page = await getEntryBySlug("pages", params.slug);
  return renderOgImage(page.data.title, page.data.description);
};
