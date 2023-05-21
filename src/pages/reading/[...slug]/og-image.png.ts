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
  const posts = await getCollection("reading");
  return posts.map((book) => ({
    params: { slug: book.slug },
    props: book,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  const book = await getEntryBySlug("reading", params.slug);

  const date = book.data.dateRead.toLocaleDateString("en-us", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });

  const cooperBtData = await cooperBt;
  const robotoData = await roboto;
  const base64Pattern = (await pattern).toString("base64");

  // FIXME: We shouldn't require a static path for the cover images
  const coverImage = await fs.readFile(path.join("./src/content/reading", params.slug, "_cover.jpg"))

  const coverImageRatio = book.data.cover.width / book.data.cover.height;
  const base64CoverImage = coverImage.toString("base64");

  let coverImageMimeType = "image/jpeg";
  if (book.data.cover.format == "png") {
    coverImageMimeType = "image/png";
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
                    children: book.data.title,
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
                    children: `${book.data.author}`,
                    style: {
                      color: "#333",
                      fontFamily: "Roboto",
                      fontSize: "40px",
                    },
                  },
                },
                {
                  type: "span",
                  props: {
                    children: `Read on ${date}`,
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
                width: `${1200-160-coverImageRatio*450-20}px`,
              },
            },
          },
          {
            type: "div",
            props: {
              style: {
                width: `${coverImageRatio * 450}px`,
                height: "450px",
                marginLeft: "20px",
                backgroundImage: `url('data:${coverImageMimeType};base64,${base64CoverImage}')`,
                backgroundSize: `${coverImageRatio * 450}px 450px`,
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
