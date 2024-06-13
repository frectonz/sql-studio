import { z } from "zod";
import { createZodFetcher } from "zod-fetch";

const BASE_URL = import.meta.env.PROD ? "/api" : "http://localhost:3030/api";

const overview = z.object({
  file_name: z.string(),
  sqlite_version: z.string(),
  file_size: z.string(),
  created: z
    .string()
    .datetime()
    .transform((x) => new Date(x))
    .nullable(),
  modified: z
    .string()
    .datetime()
    .transform((x) => new Date(x)),
  tables: z.number(),
  indexes: z.number(),
  triggers: z.number(),
  views: z.number(),
  counts: z
    .object({
      name: z.string(),
      count: z.number(),
    })
    .array(),
});

const tables = z.object({
  tables: z
    .object({
      name: z.string(),
      count: z.number(),
    })
    .array(),
});

const table = z.object({
  name: z.string(),
  sql: z.string(),
  row_count: z.number(),
  index_count: z.number(),
  table_size: z.string(),
  columns: z.string().array(),
  rows: z.any().array().array(),
});

const query = z.object({
  columns: z.string().array(),
  rows: z.any().array().array(),
});

const $fetch = createZodFetcher();

export const fetchOverview = () => $fetch(overview, `${BASE_URL}/`);
export const fetchTables = () => $fetch(tables, `${BASE_URL}/tables`);
export const fetchTable = (name: string) =>
  $fetch(table, `${BASE_URL}/tables/${name}`);
export const fetchQuery = (value: string) =>
  $fetch(query, `${BASE_URL}/query`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query: value }),
    credentials: "omit",
  });
