import renderOgImage from "../../shared/renderOgImage.ts";
import { BLOG_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage("Blog", BLOG_DESCRIPTION);
};
