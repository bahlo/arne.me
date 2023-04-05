import renderOgImage from "../../shared/renderOgImage.ts";
import { BOOKS_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage("Books", BOOKS_DESCRIPTION);
};
