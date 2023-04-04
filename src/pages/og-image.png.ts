import satori from "satori";
import fs from "fs/promises";
import sharp from "sharp";
import { getCollection, getEntryBySlug } from "astro:content";
import sizeOf from "image-size";

const cooperBtBold = fs.readFile(
  "./public/fonts/cooperbt/1296123/d6b715ec-1259-4329-9cfe-5e9d545eea39.woff"
);
const cooperBt = fs.readFile(
  "./public/fonts/cooperbt/1296125/20e6a666-635b-4b16-9068-d92fc0a1f0ec.woff"
);
const pattern = fs.readFile("./public/dot-grid.png");

export const get: APIRoute = async function get({ params, request }) {
  const cooperBtBoldData = await cooperBtBold;
  const cooperBtData = await cooperBt;
  const base64Pattern = (await pattern).toString("base64");

  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "h1",
            props: {
              children: "Hey, I’m Arne —",
              style: {
                fontFamily: "Cooper BT Bold",
                fontWeight: "bold",
                fontSize: "74px",
                lineHeight: 1,
                color: "#111",
                marginBottom: "20px",
              },
            },
          },
          {
            type: "span",
            props: {
              children:
                "a full stack software engineer, team lead, podcaster & dad based near Frankfurt, Germany.",
              style: {
                fontFamily: "Cooper BT",
                fontWeight: "normal",
                fontSize: "48px",
                lineHeight: 1.2,
                color: "rgba(0, 0, 0, .7)",
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
          name: "Cooper BT Bold",
          data: cooperBtBoldData,
          weight: "bold",
          style: "normal",
        },
        {
          name: "Cooper BT",
          data: cooperBtData,
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
