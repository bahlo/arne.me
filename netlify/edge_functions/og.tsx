import React from "https://esm.sh/react@18.2.0";
import { ImageResponse } from "https://deno.land/x/og_edge@0.0.2/mod.ts";

export default function handler(req: Request) {
  // Get the query parameters from the request
  const url = new URL(req.url);
  const params = new URLSearchParams(url.search);
  const title = params.get("title") ?? "Created with Netlify edge functions";
  const pubDate = params.get("pubDate") ?? new Date().toISOString();

  // Generate the open graph image
  return new ImageResponse((
    <div
    style={{
      height: '100%',
      width: '100%',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      backgroundColor: '#fff',
      fontSize: 32,
    }}
    >
      <div>{title}</div>
      <div>{pubDate}</div>
    </div>
  ));
}
