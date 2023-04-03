import satori from "satori";
import fs from "fs/promises";
import type { APIRoute } from "astro";
import sharp from "sharp";
import { getCollection, getEntryBySlug } from "astro:content";

export async function getStaticPaths() {
  const issues = await getCollection("weekly");
  return issues.map((issue) => ({
    params: { num: issue.slug },
    props: issue,
  }));
}

const cooperBt = fs.readFile(
  "./public/fonts/cooperbt/1296123/d6b715ec-1259-4329-9cfe-5e9d545eea39.woff"
);
const roboto = fs.readFile("./public/fonts/roboto/Roboto-Regular.ttf");
const pattern = fs.readFile("./public/dot-grid.png");

export const get: APIRoute = async function get({ params, request }) {
  const cooperBtData = await cooperBt;
  const robotoData = await roboto;
  const base64Pattern = (await pattern).toString("base64");

  const issue = await getEntryBySlug("weekly", params.num);

  const date = issue.data.date.toLocaleDateString("en-us", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });

  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "h1",
            props: {
              children: issue.data.title,
              style: {
                fontFamily: "Cooper BT",
                fontSize: "64px",
                lineHeight: 1,
                color: "#111",
                marginBottom: "16px",
              },
            },
          },
          {
            type: "span",
            props: {
              children: `Issue #${params.num} of Arne's Weekly from ${date}.`,
              style: {
                color: "#333",
                fontFamily: "Roboto",
                fontSize: "32px",
              },
            },
          },
        ],
        style: {
          width: 1200,
          height: 630,
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          borderBottom: "20px solid rgb(53, 88, 255)",
          backgroundImage: `url('data:image/png;base64,${base64Pattern}')`,
          backgroundRepeat: "repeat",
          padding: "80px",
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
