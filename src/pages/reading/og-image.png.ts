import renderOgImage from "../../shared/renderOgImage.ts";
import { READING_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage("Reading", READING_DESCRIPTION);
};
