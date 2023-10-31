---
title: "Static OG (Open Graph) Images in Astro"
description: "A guide to set up build-time Open Graph images in Astro using Satori, sharp and Astro endpoints."
published: "2023-04-07"
location: "Frankfurt, Germany"
---

Have you every shared a link to a friend and wondered where the image preview 
came from?
That's the [Open Graph Protocol](https://ogp.me/), a set of HTML `<meta>` 
extensions that can enrich a link, originally invented at Facebook.
This blog post describes how you can generate these images on build time in 
the [Astro web framework](https://astro.build).

<!-- more -->

Most of the guides out there (including Vercel's official [OG Image Generation](https://vercel.com/docs/concepts/functions/edge-functions/og-image-generation)) 
use a function to generate the OG image dynamically.
While there's nothing wrong with that, I really wanted mine to be statically
generated on build-time; it's faster, cheaper and cooler.

## Build your image

The first thing to do is figure out what your OG image should look like.
We're going to use Vercel's [Satori](https://github.com/vercel/satori), a 
JavaScript library which can render a HTML tree as SVG.
Vercel also has a great [OG Image Playground](https://og-playground.vercel.app/), 
where you can play around and quickly get results.

![A screenshot of the OG Image Playground showing a code editor on the left and a preview and some settings on the right.](/articles/static-og-images-in-astro/og-image-playground.png)

Set the size to 1200&times;630px as that's what Facebook 
recommends in [their guidelines](https://developers.facebook.com/docs/sharing/webmasters/images/).
Use Flexbox liberally and enable debug mode when figuring out the layout.
[A Complete Guide to Flexbox](https://css-tricks.com/snippets/css/a-guide-to-flexbox/) 
is a great resource to have at hand.

## Create an Astro endpoint

Got a nice image built in the playground? Then let's get started.
Install Satori to generate the SVG and [sharp](https://github.com/lovell/sharp) 
to then convert it to PNG:

```sh
$ npm install satori sharp
```

Then create an endpoint, e.g. `pages/og-image.png.ts` with the following code:

```typescript
import fs from "fs/promises";
import satori from "satori";
import sharp from "sharp";
import type { APIRoute } from 'astro';

export const get: APIRoute = async function get({ params, request }) {
  const robotoData = await fs.readFile("./public/fonts/roboto/Roboto-Regular.ttf");

  const svg = await satori(
    { 
      type: "h1", 
      props: { 
        children: "Hello world", 
        style: { 
          fontWeight: "bold" 
        }
      }
    },
    {
      width: 1200,
      height: 630,
      fonts: [
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
```

One drawback that you can already see in the code above is that Astro does not support TSX endpoints, so we'll need to use React-elements-like objects.

You'll also always need to provide a font because it'll be embedded.
I used [Roboto](https://fonts.google.com/specimen/Roboto) in the example above,
[Inter](https://rsms.me/inter/) or [Open Sans](https://fonts.google.com/specimen/Open+Sans) 
are other solid free sans fonts (choose WOFF or TTF/OTF, WOFF2 is not supported).

Run `astro dev` and navigate to the endpoint (in our example [:3000/og-image.png](http://localhost:3000/og-image.png))
to see the generated image.

## Generate for each item in a collection

Once you have an image that you like in a function, you can create them in 
batch, for whole collections. 
Let's assume you have a `blog` collection and your blog posts live at `pages/blog/:slug/index.astro`.
Create a `pages/blog/:slug/og-image.png.ts` with your API route and the `getStaticPaths` function exported:

```typescript
import { getCollection, getEntryBySlug } from "astro:content";

export async function getStaticPaths() {
  const posts = await getCollection("blog");
  return posts.map((post) => ({
    params: { slug: post.slug },
    props: post,
  }));
}

export const get: APIRoute = async function get({ params, request }) {
  // ...
}
```

If you now run `astro build`, you'll see that it statically generates an OG image for every blog post you have on your site.

## Handling images

Images (like fonts) need to be embedded into the SVG. 
The easiest way I found is using [data urls](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs)
by first reading the file to a Base64 string like this:

```typescript
const myImageBase64 = (await fs.readFile("./public/my-image.png")).toString("base64");
```

And then setting it as the `backgroundImage` or `src` property in Satori:
```typescript
{
  type: "div",
  props: {
    style: {
      backgroundImage: `url('data:image/png;base64,${myImageBase64}')`,
    }
  }
}
```

Be aware that Satori does not support `backgroundSize: cover`, so if you have 
that use case, you'll need to build it yourself with [image-size](https://github.com/image-size/image-size) and some math.

## Set OG tags in HTML

Now the only thing left to do is link to your OG images in your `<head>`.
There are two properties you'll want to use for images: `og:image` and 
`twitter:image`.

```html
<meta property="og:image" content="/blog/og-image.png" />
<meta property="twitter:image" content="/blog/og-image.png" />
```

Use dynamic paths on collections to automatically use the correct image.
Check out [the Open Graph protocol](https://ogp.me/) for more Open Graph meta 
extensions.

## Further links & conclusion

If you want to see real, working code (I know I often do), check out the endpoint
that powers the OG images of my book reviews:
[`pages/books/[...slug]/og-image.png.ts`](https://github.com/bahlo/arne.me/blob/main/src/pages/books/%5B...slug%5D/og-image.png.ts). 
This is what it looks like: [OG image of a book review](/books/the-design-of-everyday-things/og-image.png).

Did I miss anything? Can this workflow be further improved?
[Drop me an email](mailto:hey@arne.me) or [@ me in the Fediverse](https://spezi.social/@arne), 
I'd love to hear from you.
