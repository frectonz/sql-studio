import type { Metadata } from "next";
import Link from "next/link";
import { CopyButton } from "./copy-button";

export const metadata: Metadata = {
  title: "SQL Studio — Single Binary SQL Database Explorer",
  description:
    "Single binary SQL database explorer. 10 databases. One interface. No config files.",
  openGraph: {
    title: "SQL Studio",
    description:
      "Single binary SQL database explorer. 10 databases. One interface.",
    images: [{ url: "/og/index.png", width: 1200, height: 630 }],
  },
  twitter: {
    card: "summary_large_image",
    title: "SQL Studio",
    description:
      "Single binary SQL database explorer. 10 databases. One interface.",
    images: ["/og/index.png"],
  },
};

const DATABASES = [
  { name: "SQLite", command: "sqlite [file]", description: "Local .db files" },
  { name: "PostgreSQL", command: "postgres [url]", description: "PostgreSQL servers" },
  { name: "MySQL", command: "mysql [url]", description: "MySQL & MariaDB" },
  { name: "DuckDB", command: "duckdb [file]", description: "Analytics files" },
  { name: "libSQL", command: "libsql [url] [token]", description: "Remote Turso" },
  { name: "ClickHouse", command: "clickhouse [...]", description: "Analytics servers" },
  { name: "MSSQL", command: "mssql [conn]", description: "SQL Server" },
  { name: "Parquet", command: "parquet [file]", description: "Columnar files" },
  { name: "CSV", command: "csv [file]", description: "CSV files" },
  { name: "Local libSQL", command: "local-libsql [db]", description: "Local libSQL" },
];

const FEATURES = [
  {
    title: "OVERVIEW",
    description: "Database metadata, row counts, index statistics, and bar charts at a glance",
    image: "/overview.png",
  },
  {
    title: "TABLES",
    description: "Browse tables with metadata cards, creation SQL, and infinite-scroll data grids",
    image: "/tables.png",
  },
  {
    title: "QUERY",
    description: "Monaco-powered SQL editor with IntelliSense, auto-execute, and configurable timeouts",
    image: "/query.png",
  },
  {
    title: "ERD",
    description: "Interactive entity-relationship diagrams with foreign key visualization",
    image: "/erd.png",
  },
];

const INSTALL_METHODS = [
  {
    label: "SHELL",
    command: "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/frectonz/sql-studio/releases/download/0.1.50/sql-studio-installer.sh | sh",
  },
  { label: "NIX", command: "nix shell nixpkgs#sql-studio" },
  { label: "DOCKER", command: "docker run -p 3030:3030 frectonz/sql-studio" },
];

function DatabaseIcon({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <ellipse cx="12" cy="5" rx="9" ry="3" />
      <path d="M3 5V19A9 3 0 0 0 21 19V5" />
      <path d="M3 12A9 3 0 0 0 21 12" />
    </svg>
  );
}

function ArrowIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5 12h14" />
      <path d="m12 5 7 7-7 7" />
    </svg>
  );
}


export default function LandingPage() {
  return (
    <div className="ld-mono ld-bg" style={{ color: "var(--ss-foreground)", minHeight: "100vh" }}>
      {/* NAV */}
      <nav style={{ borderBottom: "2px solid var(--ss-border)" }}>
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
          <Link href="/" className="flex items-center gap-2 text-sm font-bold tracking-tight" style={{ color: "var(--ss-primary)" }}>
            <DatabaseIcon />
            SQL STUDIO
          </Link>
          <div className="flex items-center gap-6 text-xs tracking-widest" style={{ color: "var(--ss-muted)" }}>
            <Link href="/docs" className="transition-colors duration-100 hover:text-white">
              DOCS
            </Link>
            <Link href="https://github.com/frectonz/sql-studio" className="transition-colors duration-100 hover:text-white" target="_blank" rel="noopener noreferrer">
              GITHUB
            </Link>
          </div>
        </div>
      </nav>

      {/* HERO */}
      <section className="relative overflow-hidden" style={{ borderBottom: "2px solid var(--ss-border)" }}>
        <div className="ld-watermark" style={{ fontSize: "40vw", top: "50%", left: "50%", transform: "translate(-50%, -50%)" }}>
          SELECT&nbsp;*
        </div>
        <div className="relative z-10 mx-auto grid max-w-6xl grid-cols-1 lg:grid-cols-[2fr_3fr]">

          {/* Left — text */}
          <div className="px-6 pb-16 pt-20 md:pb-24 md:pt-28" style={{ borderRight: "none" }}>
            <div className="ld-up mb-6 inline-block border px-3 py-1 text-xs font-bold tracking-widest uppercase" style={{ animationDelay: "0.1s", color: "var(--ss-primary)", borderColor: "var(--ss-primary)" }}>
              V0.1.50
            </div>

            <h1 className="ld-up font-black text-6xl sm:text-7xl md:text-8xl lg:text-9xl" style={{ animationDelay: "0.2s", lineHeight: 0.85, letterSpacing: "-0.04em" }}>
              SQL<br /><span style={{ color: "var(--ss-primary)" }}>Studio</span>
            </h1>

            <p className="ld-up mt-8 max-w-md text-sm leading-relaxed" style={{ animationDelay: "0.35s", color: "var(--ss-muted)" }}>
              Single binary SQL database explorer. 10 databases. One interface. No config files.
            </p>

            <div className="ld-up mt-8 flex flex-wrap gap-3" style={{ animationDelay: "0.45s" }}>
              <Link href="/docs/installation" className="inline-flex items-center gap-2 border px-5 py-2.5 text-sm font-bold uppercase tracking-wider transition-all duration-100 hover:translate-x-[-1px] hover:translate-y-[-1px] hover:shadow-[2px_2px_0_hsl(159,100%,50%)]" style={{ background: "var(--ss-primary)", color: "hsl(159, 10%, 5%)", borderColor: "var(--ss-primary)" }}>
                GET STARTED
                <ArrowIcon />
              </Link>
              <Link href="https://sql-studio.frectonz.et/" className="inline-flex items-center gap-2 border px-5 py-2.5 text-sm font-bold uppercase tracking-wider transition-all duration-100 hover:translate-x-[-1px] hover:translate-y-[-1px] hover:shadow-[2px_2px_0_hsl(159,50%,15%)]" style={{ borderColor: "var(--ss-border)", color: "var(--ss-muted)" }} target="_blank" rel="noopener noreferrer">
                LIVE PREVIEW
                <ArrowIcon />
              </Link>
            </div>


          </div>

          {/* Right — hero image with 3D tilt */}
          <div className="ld-up flex items-center justify-center pb-16 pt-8 lg:pb-24 lg:pt-28" style={{ animationDelay: "0.5s", perspective: "1200px" }}>
            <div className="ld-hero-tilt" style={{ transformStyle: "preserve-3d", transform: "rotateY(-8deg) rotateX(4deg)", width: "100%" }}>
              <img src="/overview.png" alt="SQL Studio overview dashboard" className="block w-full object-contain" />
            </div>
          </div>
        </div>
      </section>

      {/* DATABASES */}
      <section className="relative overflow-hidden" style={{ borderBottom: "2px solid var(--ss-border)" }}>
        <div className="ld-watermark" style={{ fontSize: "25vw", bottom: "-5%", right: "-3%" }}>
          SELECT&nbsp;*
        </div>
        <div className="relative z-10 mx-auto max-w-6xl px-6 py-20 md:py-28">
          <div className="ld-scroll mb-2">
            <span className="text-xs font-bold tracking-widest" style={{ color: "var(--ss-primary)" }}>
              [01] COMPATIBILITY
            </span>
          </div>
          <h2 className="ld-scroll mb-4 text-3xl font-black uppercase tracking-tighter md:text-5xl">
            10 DATABASES.<br />ONE TOOL.
          </h2>
          <p className="ld-scroll mb-12 max-w-md text-sm" style={{ color: "var(--ss-muted-dim)" }}>
            From local SQLite files to remote PostgreSQL clusters. Same interface, same command.
          </p>

          <div className="grid grid-cols-2 gap-0 sm:grid-cols-3 md:grid-cols-5" style={{ border: "1px solid var(--ss-border)" }}>
            {DATABASES.map((db, i) => (
              <div key={db.name} className="ld-card border-0 px-4 py-4" style={{ borderRight: (i + 1) % 5 !== 0 ? "1px solid var(--ss-border)" : "none", borderBottom: i < 5 ? "1px solid var(--ss-border)" : "none" }}>
                <div className="mb-2 text-xs font-black uppercase tracking-wider" style={{ color: "var(--ss-foreground)" }}>
                  {db.name}
                </div>
                <div className="mb-1.5 text-xs" style={{ color: "var(--ss-primary)" }}>
                  {db.command}
                </div>
                <div className="text-xs" style={{ color: "var(--ss-muted-dim)" }}>
                  {db.description}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* FEATURES */}
      <section className="relative overflow-hidden" style={{ borderBottom: "2px solid var(--ss-border)" }}>
        <div className="ld-watermark" style={{ fontSize: "30vw", top: "10%", left: "-5%" }}>
          JOIN
        </div>
        <div className="relative z-10 mx-auto max-w-6xl px-6 py-20 md:py-28">
          <div className="ld-scroll mb-2">
            <span className="text-xs font-bold tracking-widest" style={{ color: "var(--ss-primary)" }}>
              [02] FEATURES
            </span>
          </div>
          <h2 className="ld-scroll mb-4 text-3xl font-black uppercase tracking-tighter md:text-5xl">
            EVERYTHING YOU NEED<br />TO EXPLORE.
          </h2>
          <p className="ld-scroll mb-12 max-w-md text-sm" style={{ color: "var(--ss-muted-dim)" }}>
            Overview dashboards, table browsers, a full SQL editor, and interactive ERD diagrams.
          </p>

          <div className="grid grid-cols-1 gap-4 md:grid-cols-12">
            {FEATURES.map((feature, i) => {
              const span = i === 0 || i === 3 ? "md:col-span-7" : "md:col-span-5";
              return (
                <div key={feature.title} className={`ld-feature ld-scroll group ${span}`}>
                  <div className="p-3 md:p-4">
                    <img src={feature.image} alt={feature.title} className="w-full object-contain" loading="lazy" />
                  </div>
                  <div className="border-t px-4 py-3 md:px-5 md:py-4" style={{ borderColor: "var(--ss-border)" }}>
                    <div className="mb-1.5 inline-block border px-2 py-0.5 text-xs font-black tracking-widest" style={{ borderColor: "var(--ss-primary)", color: "var(--ss-primary)" }}>
                      {feature.title}
                    </div>
                    <p className="mt-1.5 text-xs leading-relaxed" style={{ color: "var(--ss-muted)" }}>
                      {feature.description}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* INSTALL */}
      <section style={{ borderBottom: "2px solid var(--ss-border)" }}>
        <div className="mx-auto max-w-6xl px-6 py-20 md:py-28">
          <div className="ld-scroll mb-2">
            <span className="text-xs font-bold tracking-widest" style={{ color: "var(--ss-primary)" }}>
              [03] INSTALL
            </span>
          </div>
          <h2 className="ld-scroll mb-4 text-3xl font-black uppercase tracking-tighter md:text-5xl">
            GET STARTED<br />IN SECONDS.
          </h2>
          <p className="ld-scroll mb-12 max-w-md text-sm" style={{ color: "var(--ss-muted-dim)" }}>
            No configuration files. No dependencies to manage.
          </p>

          <div className="grid grid-cols-1 gap-4">
            {INSTALL_METHODS.map((method) => (
              <div key={method.label} className="ld-install ld-scroll">
                <div className="flex items-center justify-between border-b px-4 py-2" style={{ borderColor: "var(--ss-border)" }}>
                  <span className="text-xs font-black tracking-widest" style={{ color: "var(--ss-muted-dim)" }}>
                    {method.label}
                  </span>
                  <CopyButton text={method.command} />
                </div>
                <div className="px-4 py-4">
                  <code className="block whitespace-pre-wrap break-all text-xs leading-relaxed" style={{ color: "var(--ss-primary)" }}>
                    $ {method.command}
                  </code>
                </div>
              </div>
            ))}
          </div>

          <div className="ld-scroll mt-10" style={{ color: "var(--ss-muted-dim)" }}>
            <p className="text-sm">
              Or try the built-in preview:{" "}
              <code className="border px-2 py-0.5 text-xs" style={{ borderColor: "var(--ss-border)", color: "var(--ss-primary)" }}>
                sql-studio sqlite preview
              </code>
            </p>
          </div>
        </div>
      </section>

      {/* FOOTER */}
      <footer className="px-6 py-12">
        <div className="mx-auto flex max-w-6xl flex-col items-center gap-5 md:flex-row md:justify-between">
          <Link href="/" className="flex items-center gap-2 text-sm font-bold tracking-tight" style={{ color: "var(--ss-primary)" }}>
            <DatabaseIcon size={16} />
            SQL STUDIO
          </Link>
          <div className="flex gap-6 text-xs tracking-widest" style={{ color: "var(--ss-muted-dim)" }}>
            <Link href="/docs" className="transition-colors duration-100 hover:text-white">DOCS</Link>
            <Link href="https://github.com/frectonz/sql-studio" className="transition-colors duration-100 hover:text-white" target="_blank" rel="noopener noreferrer">GITHUB</Link>
            <Link href="https://github.com/frectonz/sql-studio/releases" className="transition-colors duration-100 hover:text-white" target="_blank" rel="noopener noreferrer">RELEASES</Link>
          </div>
          <span className="text-xs" style={{ color: "var(--ss-border)" }}>RUST + REACT</span>
        </div>
      </footer>
    </div>
  );
}
