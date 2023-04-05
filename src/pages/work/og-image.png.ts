import renderOgImage from "../../shared/renderOgImage.ts";
import { WORK_DESCRIPTION } from "../../consts.ts";

export const get: APIRoute = async function get({ params, request }) {
  return renderOgImage("Work", WORK_DESCRIPTION);
};
