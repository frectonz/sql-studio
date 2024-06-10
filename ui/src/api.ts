import { createFetch } from "@better-fetch/fetch";
import type { FetchSchema, Strict } from "@better-fetch/fetch/typed";
import { z } from "zod";

const routes = {
  "/": {
    output: z.object({
      file_name: z.string(),
      sqlite_version: z.string(),
      file_size: z.string(),
      created: z.date(),
      modified: z.date(),
      tables: z.number(),
      indexes: z.number(),
      triggers: z.number(),
      views: z.number(),
    }),
  },
} satisfies FetchSchema;

export const $fetch = createFetch<Strict<typeof routes>>({
  baseURL: "http://localhost:3030/api",
  throw: true,
});
