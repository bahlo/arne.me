import renderOgImage from "../../shared/renderOgImage.ts";
import { PROJECTS_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage("Projects", PROJECTS_DESCRIPTION);
};
