import satori from "satori";
import fs from "fs/promises";
import sharp from "sharp";

const cooperBt = fs.readFile(
  "./public/fonts/cooperbt/1296123/d6b715ec-1259-4329-9cfe-5e9d545eea39.woff"
);
const roboto = fs.readFile("./public/fonts/roboto/Roboto-Regular.ttf");
const pattern = fs.readFile("./public/dot-grid.png");

export default async function renderOgImage(
  title: string,
  description: string
) {
  const cooperBtData = await cooperBt;
  const robotoData = await roboto;
  const base64Pattern = (await pattern).toString("base64");

  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "h1",
            props: {
              children: title,
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
              children: description,
              style: {
                color: "#333",
                fontFamily: "Roboto",
                fontSize: "40px",
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
}
