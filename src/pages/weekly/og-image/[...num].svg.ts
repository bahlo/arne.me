import satori from 'satori';
import fs from 'fs/promises';
import type { APIRoute } from 'astro';
import { getCollection, getEntryBySlug } from 'astro:content';

export async function getStaticPaths() {
  const issues = await getCollection('weekly');
  return issues.map((issue) => ({
    params: { num: issue.slug },
    props: issue,
  }));
}

const cooperBt = fs.readFile('./public/fonts/cooperbt/1296123/d6b715ec-1259-4329-9cfe-5e9d545eea39.woff');
const roboto = fs.readFile('./public/fonts/roboto/Roboto-Regular.ttf');

export const get: APIRoute = async function get({ params, request }) {
  const cooperBtData = await cooperBt;
  const robotoData = await roboto;

  const issue = await getEntryBySlug('weekly', params.num);

  const date = issue.data.date.toLocaleDateString('en-us', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });

  const svg = await satori(
    {
      type: 'div',
      props: {
        children: [
          {
            type: 'div',
            props: {
              children: [
                {
                  type: 'h1',
                  props: {
                    children: issue.data.title,
                    style: { 
                      color: '#111',
                      fontFamily: 'Cooper BT',
                      fontSize: '72px',
                      lineHeight: 1,
                    },
                  },
                },
                {
                  type: 'span',
                  props: {
                    children: `Issue #${params.num} of Arne's Weekly from ${date}.`,
                    style: { 
                      color: '#333',
                      fontFamily: 'Roboto',
                      fontSize: '40px',
                    },
                  }
                },
              ],
              style: {
                display: 'flex',
                flexDirection: 'column',
                flexGrow: 1,
                justifyContent: 'center',
                padding: '32px',
              }
            }
          },
          {
            type: 'div',
            props: {
              children: 'Arne\'s Weekly is my newsletter, where I collect interesting stories from all over the web every Sunday.',
              style: {
                padding: '32px',
                background: "rgba(53, 88, 255, 0.04)",
                color: '#111',
                fontFamily: 'Roboto',
                fontSize: '40px',
                lineHeight: 1.4,
                borderTop: '2px solid rgba(53, 88, 255, 0.04)',
              }
            }
          }
        ],
        style: { 
          width: 1200,
          height: 630,
          display: "flex",
          flexDirection: "column",
          background: "rgba(53, 88, 255, 0.01)",
        },
      },
    },
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: 'Cooper BT',
          data: cooperBtData,
          weight: 'bold',
          style: 'normal',
        },
        {
          name: 'Roboto',
          data: robotoData,
          weight: 'normal',
          style: 'normal',
        },
      ],
    },
  );

  return new Response(svg, {
    headers: {
      'Content-Type': 'image/svg+xml',
    },
  });
}
