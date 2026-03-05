import { defineDocs } from "@farming-labs/docs";
import {
  BookOpen,
  Rocket,
  Terminal,
  Database,
  Settings,
  Layout,
  Table,
  Code,
  GitBranch,
  HardDrive,
  Server,
  FileSpreadsheet,
  FileText,
} from "lucide-react";
import { greentree } from "@farming-labs/theme/greentree";

export default defineDocs({
  entry: "docs",
  github: {
    url: "https://github.com/frectonz/sql-studio",
    branch: "main",
    directory: "docs",
  },
  theme: greentree({
    ui: {
      layout: {
        toc: { enabled: true, depth: 3, style: "default" },
        sidebarWidth: 280,
      },
      sidebar: { style: "default" },
      typography: {
        font: {
          style: {
            sans: "var(--font-geist-sans, system-ui, -apple-system, sans-serif)",
            mono: "var(--font-geist-mono, ui-monospace, monospace)",
          },
          h1: { size: "2.25rem", weight: 700, letterSpacing: "-0.025em" },
          h2: { size: "1.5rem", weight: 600, letterSpacing: "-0.015em" },
          h3: { size: "1.25rem", weight: 600 },
          body: { size: "0.975rem", lineHeight: "1.8" },
        },
      },
    },
  }),
  nav: {
    title: (
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <Database size={14} />
        <span className="uppercase font-mono tracking-tighter">SQL Studio</span>
      </div>
    ),
    url: "/",
  },
  icons: {
    book: <BookOpen size={16} />,
    rocket: <Rocket size={16} />,
    terminal: <Terminal size={16} />,
    database: <Database size={16} />,
    settings: <Settings size={16} />,
    layout: <Layout size={16} />,
    table: <Table size={16} />,
    code: <Code size={16} />,
    gitbranch: <GitBranch size={16} />,
    harddrive: <HardDrive size={16} />,
    server: <Server size={16} />,
    spreadsheet: <FileSpreadsheet size={16} />,
    file: <FileText size={16} />,
  },

  sidebar: { flat: true },
  breadcrumb: { enabled: true },

  ordering: [
    { slug: "" },
    { slug: "installation" },
    { slug: "quick-start" },
    {
      slug: "databases",
      children: [
        { slug: "sqlite" },
        { slug: "libsql" },
        { slug: "local-libsql" },
        { slug: "postgresql" },
        { slug: "mysql" },
        { slug: "duckdb" },
        { slug: "parquet" },
        { slug: "csv" },
        { slug: "clickhouse" },
        { slug: "mssql" },
      ],
    },
    { slug: "configuration" },
    {
      slug: "features",
      children: [
        { slug: "overview" },
        { slug: "table-explorer" },
        { slug: "query-editor" },
        { slug: "erd-viewer" },
      ],
    },
  ],
  metadata: {
    titleTemplate: "%s – SQL Studio",
    description:
      "SQL Studio — single binary, single command SQL database explorer.",
  },
  themeToggle: {
    enabled: true,
    default: "dark",
  },
});
