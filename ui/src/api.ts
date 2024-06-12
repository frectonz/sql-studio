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
    .transform((x) => new Date(x)),
  modified: z
    .string()
    .datetime()
    .transform((x) => new Date(x)),
  tables: z.number(),
  indexes: z.number(),
  triggers: z.number(),
  views: z.number(),
  counts: z.array(
    z.object({
      name: z.string(),
      count: z.number(),
    }),
  ),
});

const tables = z.object({
  tables: z.array(
    z.object({
      name: z.string(),
      count: z.number(),
    }),
  ),
});

const table = z.object({
  name: z.string(),
  sql: z.string(),
  row_count: z.number(),
  table_size: z.string(),
  indexes: z.array(z.string()),
  columns: z.array(z.string()),
  rows: z.array(z.array(z.any())),
});

const $fetch = createZodFetcher();

export const fetchOverview = () => $fetch(overview, `${BASE_URL}/`);
export const fetchTables = () => $fetch(tables, `${BASE_URL}/tables`);
export const fetchTable = (name: string) =>
  $fetch(table, `${BASE_URL}/tables/${name}`);
