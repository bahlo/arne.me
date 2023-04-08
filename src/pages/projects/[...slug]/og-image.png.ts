import satori from "satori";
import fs from "fs/promises";
import path from "path";
import sharp from "sharp";
import { getCollection, getEntryBySlug } from "astro:content";

const cooperBt = fs.readFile(
  "./public/fonts/cooperbt/1296123/d6b715ec-1259-4329-9cfe-5e9d545eea39.woff"
);
const roboto = fs.readFile("./public/fonts/roboto/Roboto-Regular.ttf");
const pattern = fs.readFile("./public/dot-grid.png");

export async function getStaticPaths() {
  const projects = await getCollection("projects");
  return projects.map((project) => ({
    params: { slug: project.slug },
    props: project,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  const project = await getEntryBySlug("projects", params.slug);

  const cooperBtData = await cooperBt;
  const robotoData = await roboto;
  const base64Pattern = (await pattern).toString("base64");

  // FIXME: We shouldn't require a static path for the cover images
  const image = await fs.readFile(path.join("./src/content/projects", params.slug, "_cover.png"))

  const imageRatio = project.data.image.width / project.data.image.height;
  const base64Image = image.toString("base64");

  let imageMimeType = "image/jpeg";
  if (project.data.image.format == "png") {
    imageMimeType = "image/png";
  }
    
  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "div",
            props: {
              children: [
                {
                  type: "h1",
                  props: {
                    children: project.data.title,
                    style: {
                      fontFamily: "Cooper BT",
                      fontSize: "64px",
                      lineHeight: 1,
                      color: "#111",
                      marginBottom: "32px",
                    },
                  },
                },
                {
                  type: "span",
                  props: {
                    children: `${project.data.type} · Est. ${project.data.est}`,
                    style: {
                      color: "#333",
                      fontFamily: "Roboto",
                      fontSize: "40px",
                    },
                  },
                },
              ],
              style: {
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                width: `${1200-160-imageRatio*320-20}px`,
              },
            },
          },
          {
            type: "div",
            props: {
              style: {
                width: `${imageRatio * 320}px`,
                height: "320px",
                marginLeft: "20px",
                backgroundImage: `url('data:${imageMimeType};base64,${base64Image}')`,
                backgroundSize: `${imageRatio * 320}px 320px`,
                backgroundRepeat: "no-repeat",
              }
            }
          }
        ],
        style: {
          width: 1200,
          height: 630,
          display: "flex",
          borderBottom: "20px solid rgb(53, 88, 255)",
          backgroundImage: `url('data:image/png;base64,${base64Pattern}')`,
          backgroundRepeat: "repeat",
          padding: "80px",
          alignItems: "center",
        },
      },
    },
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Cooper BT",
          data: cooperBtData,
          weight: "bold",
          style: "normal",
        },
        {
          name: "Roboto",
          data: robotoData,
          weight: "normal",
          style: "normal",
        },
      ],
    }
  );

  const png = await sharp(Buffer.from(svg)).png().toBuffer();

  return new Response(png, {
    headers: {
      "Content-Type": "image/png",
    },
  });
};
