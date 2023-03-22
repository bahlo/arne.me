import { defineCollection, z } from 'astro:content';

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
    cover: z.object({
      image: z.string(),
      alt: z.string(),
      caption: z.string(),
    }).optional()
  }),
});

const weekly = defineCollection({
  schema: z.object({
    title: z.string(),
    date: z
      .string()
      .or(z.date())
      .transform((val) => new Date(val)),
  }),
});

const books = defineCollection({
  schema: z.object({
    title: z.string(),
    subtitle: z.string(),
    author: z.string(),
    rating: z.number(),
    dateRead: z
      .string()
      .optional()
      .transform((str) => (str ? new Date(str) : undefined)),
  }),
});

export const collections = { blog, weekly };
