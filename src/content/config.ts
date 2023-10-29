import { defineCollection, z } from "astro:content";

const writing = defineCollection({
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
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      date: z
        .string()
        .or(z.date())
        .transform((val) => new Date(val)),
      quoteOfTheWeek: z
        .object({
          text: z.string(),
          author: z.string(),
        })
        .optional(),
      tootOfTheWeek: z
        .object({
          text: z.string(),
          author: z.string(),
          url: z.string(),
        })
        .optional(),
      tweetOfTheWeek: z
        .object({
          text: z.string(),
          author: z.string(),
          url: z.string(),
          media: z
            .object({
              image: image(),
              alt: z.string(),
            })
            .optional(),
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
      stories: z
        .array(
          z.object({
            title: z.string(),
            url: z.string(),
            readingTimeMinutes: z.number(),
            description: z.string(),
          })
        )
        .optional(),
    }),
});

const reading = defineCollection({
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      subtitle: z.string().optional(),
      website: z.string().optional(),
      author: z.string(),
      rating: z.number(),
      dateRead: z
        .string()
        .or(z.date())
        .transform((val) => new Date(val)),
      cover: image(),
    }),
});

const pages = defineCollection({
  schema: z.object({
    title: z.string(),
    description: z.string(),
  }),
});

const projects = defineCollection({
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      description: z.string(),
      type: z.string(),
      website: z.string().optional(),
      sortIndex: z.number(),
      est: z
        .string()
        .or(z.date())
        .transform((val) => new Date(val)),
      image: image().refine((img) => img.width >= 360, {
        message: "Cover image must be at least 360 pixels wide!",
      }),
      imageAlt: z.string(),
    }),
});

export const collections = { writing, weekly, reading, pages, projects };
