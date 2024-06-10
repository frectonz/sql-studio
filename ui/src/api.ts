import { z } from "zod";
import { createFetch } from "@better-fetch/fetch";
import type { FetchSchema } from "@better-fetch/fetch/typed";

const routes = {
  "/": {
    output: z.object({
      file_name: z.string(),
      sqlite_version: z.string(),
      file_size: z.string(),
      created: z
        .string()
        .datetime()
        .transform((x) => new Date(x)),
      modified: z
        .string()
        .datetime()
        .transform((x) => new Date(x)),
      tables: z.number(),
      indexes: z.number(),
      triggers: z.number(),
      views: z.number(),
    }),
  },
} satisfies FetchSchema;

export const $fetch = createFetch(
  {
    baseURL: "http://localhost:3030/api",
    throw: true,
  },
  routes,
);
