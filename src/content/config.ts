import { defineCollection, z } from "astro:content";

const blog = defineCollection({
  schema: z.object({
    title: z.string(),
    description: z.string(),
    pubDate: z
      .string()
      .or(z.date())
      .transform((val) => new Date(val)),
    updatedDate: z
      .string()
      .optional()
      .transform((str) => (str ? new Date(str) : undefined)),
    cover: z
      .object({
        image: z.string(),
        alt: z.string(),
        caption: z.string(),
      })
      .optional(),
  }),
});

const weekly = defineCollection({
  schema: z.object({
    title: z.string(),
    date: z
      .string()
      .or(z.date())
      .transform((val) => new Date(val)),
    tootOfTheWeek: z
      .object({
        text: z.string(),
        author: z.string(),
        url: z.string(),
      })
      .optional(),
    categories: z
      .array(
        z.object({
          title: z.string(),
          stories: z.array(
            z.object({
              title: z.string(),
              url: z.string(),
              readingTimeMinutes: z.number(),
              description: z.string(),
            })
          ),
        })
      )
      .optional(),
  }),
});

const books = defineCollection({
  schema: z.object({
    title: z.string(),
    subtitle: z.string().optional(),
    website: z.string().optional(),
    author: z.string(),
    rating: z.number(),
    dateRead: z
      .string()
      .or(z.date())
      .transform((val) => new Date(val)),
    cover: z.string(),
  }),
});

export const collections = { blog, weekly, books };
