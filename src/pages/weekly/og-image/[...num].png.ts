import { getCollection, getEntryBySlug } from "astro:content";
import renderOgImage from "../../../shared/renderOgImage.ts";

export async function getStaticPaths() {
  const issues = await getCollection("weekly");
  return issues.map((issue) => ({
    params: { num: issue.slug },
    props: issue,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  const issue = await getEntryBySlug("weekly", params.num);

  const date = issue.data.date.toLocaleDateString("en-us", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
  const description = `Arne's Weekly from ${date}.`;

  return renderOgImage(issue.data.title, description);
};
