import { z } from "zod";
import { createZodFetcher } from "zod-fetch";

let basePath = document.querySelector<HTMLMetaElement>(
  `meta[name="BASE_PATH"]`,
);
const BASE_URL = import.meta.env.PROD
  ? basePath
    ? `${basePath.content}/api`
    : "/api"
  : "http://localhost:3030/api";

const counts = z
  .object({
    name: z.string(),
    count: z.number(),
  })
  .array();

const overview = z.object({
  file_name: z.string(),
  sqlite_version: z.string().nullable(),
  db_size: z.string(),
  created: z
    .string()
    .datetime()
    .transform((x) => new Date(x))
    .nullable(),
  modified: z
    .string()
    .datetime()
    .transform((x) => new Date(x))
    .nullable(),
  tables: z.number(),
  indexes: z.number(),
  triggers: z.number(),
  views: z.number(),
  row_counts: counts,
  column_counts: counts,
  index_counts: counts,
});

const tables = z.object({
  tables: counts,
});

const table = z.object({
  name: z.string(),
  sql: z.string().nullable(),
  row_count: z.number(),
  index_count: z.number(),
  column_count: z.number(),
  table_size: z.string(),
});

const tableData = z.object({
  columns: z.string().array(),
  rows: z.any().array().array(),
});

const query = z.object({
  columns: z.string().array(),
  rows: z.any().array().array(),
});

const metadata = z.object({
  version: z.string(),
  can_shutdown: z.boolean(),
});

const $fetch = createZodFetcher();

export const fetchOverview = () => $fetch(overview, `${BASE_URL}/`);
export const fetchTables = () => $fetch(tables, `${BASE_URL}/tables`);
export const fetchTable = (name: string) =>
  $fetch(table, `${BASE_URL}/tables/${name}`);
export const fetchTableData = (name: string, page: number) =>
  $fetch(tableData, `${BASE_URL}/tables/${name}/data?page=${page}`);
export const fetchQuery = (value: string) =>
  $fetch(query, `${BASE_URL}/query`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query: value }),
  });
export const fetchMetadata = () => $fetch(metadata, `${BASE_URL}/metadata`);

export const sendShutdown = () =>
  fetch(`${BASE_URL}/shutdown`, { method: "POST" });
