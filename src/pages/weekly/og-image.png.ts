import renderOgImage from "../../shared/renderOgImage.ts";
import { WEEKLY_TITLE, WEEKLY_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage(WEEKLY_TITLE, WEEKLY_DESCRIPTION);
};
