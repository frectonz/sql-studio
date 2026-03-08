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
    command: "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/frectonz/sql-studio/releases/download/0.1.51/sql-studio-installer.sh | sh",
  },
  {
    label: "POWERSHELL",
    command: "powershell -ExecutionPolicy Bypass -c \"irm https://github.com/frectonz/sql-studio/releases/download/0.1.51/sql-studio-installer.ps1 | iex\"",
  },
  { label: "NIX", command: "nix shell nixpkgs#sql-studio" },
  { label: "DOCKER", command: "docker run -p 3030:3030 frectonz/sql-studio" },
];

function Logo({ width = 128 }: { width?: number }) {
  return (
    <svg
      style={{ width }}
      viewBox="0 0 346 81"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="173" height="81" rx="5" fill="currentColor" />
      <path
        d="M91.986 56.43C89.836 56.43 87.9727 56.0717 86.396 55.355C84.8193 54.6383 83.601 53.6207 82.741 52.302C81.9097 50.9547 81.4797 49.378 81.451 47.572H86.826C86.826 48.862 87.2847 49.8797 88.202 50.625C89.148 51.3417 90.4237 51.7 92.029 51.7C93.577 51.7 94.781 51.3417 95.641 50.625C96.5297 49.9083 96.974 48.9193 96.974 47.658C96.974 46.5973 96.6587 45.68 96.028 44.906C95.426 44.1033 94.5517 43.5587 93.405 43.272L89.793 42.283C87.3277 41.6523 85.4213 40.52 84.074 38.886C82.7553 37.252 82.096 35.274 82.096 32.952C82.096 31.1747 82.4973 29.6267 83.3 28.308C84.1027 26.9893 85.235 25.9717 86.697 25.255C88.1877 24.5383 89.9363 24.18 91.943 24.18C94.9817 24.18 97.3897 24.9683 99.167 26.545C100.944 28.093 101.847 30.1857 101.876 32.823H96.501C96.501 31.5903 96.0997 30.63 95.297 29.942C94.4943 29.2253 93.362 28.867 91.9 28.867C90.4953 28.867 89.406 29.1967 88.632 29.856C87.858 30.5153 87.471 31.447 87.471 32.651C87.471 33.7403 87.7577 34.672 88.331 35.446C88.933 36.1913 89.793 36.7217 90.911 37.037L94.652 38.069C97.146 38.6997 99.0523 39.832 100.371 41.466C101.69 43.0713 102.349 45.0637 102.349 47.443C102.349 49.2203 101.919 50.797 101.059 52.173C100.199 53.5203 98.995 54.5667 97.447 55.312C95.899 56.0573 94.0787 56.43 91.986 56.43ZM123.37 63.74L118.64 55.957L119.586 56.387C119.472 56.387 119.328 56.387 119.156 56.387C119.013 56.4157 118.855 56.43 118.683 56.43C116.677 56.43 114.928 56.043 113.437 55.269C111.947 54.495 110.786 53.4057 109.954 52.001C109.123 50.5677 108.707 48.9193 108.707 47.056V33.554C108.707 31.6333 109.123 29.985 109.954 28.609C110.786 27.2043 111.947 26.115 113.437 25.341C114.928 24.567 116.677 24.18 118.683 24.18C120.69 24.18 122.439 24.567 123.929 25.341C125.42 26.115 126.581 27.2043 127.412 28.609C128.244 30.0137 128.659 31.662 128.659 33.554V47.056C128.659 48.948 128.244 50.6107 127.412 52.044C126.61 53.4487 125.463 54.5237 123.972 55.269L129.175 63.74H123.37ZM118.683 51.7C120.174 51.7 121.306 51.2987 122.08 50.496C122.883 49.6647 123.284 48.518 123.284 47.056V33.554C123.284 32.0633 122.869 30.9167 122.037 30.114C121.235 29.2827 120.117 28.867 118.683 28.867C117.25 28.867 116.118 29.2827 115.286 30.114C114.484 30.9167 114.082 32.0633 114.082 33.554V47.056C114.082 48.518 114.484 49.6647 115.286 50.496C116.089 51.2987 117.221 51.7 118.683 51.7ZM136.995 56V24.61H142.37V51.012H155.7V56H136.995Z"
        fill="var(--ss-bg, hsl(159,10%,5%))"
      />
      <path
        d="M39.5243 38.2078C44.6766 38.2078 49.0435 37.419 52.6252 35.8414C56.2084 34.2654 58 32.3521 58 30.1016C58 27.851 56.2084 25.9377 52.6252 24.3617C49.042 22.7857 44.675 21.9985 39.5243 22C34.3735 22.0015 29.998 22.7888 26.3979 24.3617C22.7978 25.9346 20.9985 27.8487 21 30.1039C21.0015 32.3591 22.8008 34.2716 26.3979 35.8414C29.995 37.4113 34.3696 38.1993 39.5219 38.2054M39.4781 41.7688C41.352 41.7688 43.2584 41.6191 45.197 41.3197C47.1357 41.0217 48.9573 40.5803 50.6619 39.9952C52.3665 39.4102 53.8678 38.6855 55.1659 37.8211C56.464 36.9567 57.4087 35.9572 58 34.8227V41.6785C57.4087 42.8131 56.464 43.8126 55.1659 44.677C53.8694 45.5429 52.368 46.2684 50.6619 46.8534C48.9573 47.4385 47.1357 47.8799 45.197 48.1778C43.2584 48.4758 41.352 48.6247 39.4781 48.6247C37.6041 48.6247 35.6978 48.4758 33.7591 48.1778C31.8204 47.8799 30.0057 47.4385 28.315 46.8534C26.6243 46.2684 25.1306 45.5429 23.8341 44.677C22.5375 43.811 21.5928 42.8123 21 41.6809V34.8227C21.5913 35.9557 22.536 36.9551 23.8341 37.8211C25.1306 38.6855 26.6243 39.411 28.315 39.9976C30.0057 40.581 31.8204 41.0225 33.7591 41.322C35.6978 41.6214 37.6041 41.7704 39.4781 41.7688ZM39.4781 52.1881C41.352 52.1881 43.2584 52.0392 45.197 51.7412C47.1341 51.4418 48.9558 50.9995 50.6619 50.4145C52.368 49.8295 53.8694 49.1048 55.1659 48.2404C56.4625 47.3759 57.4072 46.3765 58 45.2419V52.0538C57.4087 53.1884 56.464 54.1878 55.1659 55.0522C53.8694 55.9167 52.368 56.6422 50.6619 57.2287C48.9573 57.8137 47.1357 58.2552 45.197 58.5531C43.2584 58.851 41.352 59 39.4781 59C37.6041 59 35.6978 58.851 33.7591 58.5531C31.8204 58.2552 30.0057 57.8137 28.315 57.2287C26.6243 56.6437 25.1306 55.9182 23.8341 55.0522C22.5375 54.1863 21.5928 53.1868 21 52.0538V45.2419C21.5928 46.3749 22.5375 47.3744 23.8341 48.2404C25.1306 49.1063 26.6243 49.8318 28.315 50.4168C30.0057 51.0019 31.8204 51.4433 33.7591 51.7412C35.6978 52.0392 37.6041 52.1881 39.4781 52.1881Z"
        fill="var(--ss-bg, hsl(159,10%,5%))"
      />
      <path
        d="M199.119 56.43C196.969 56.43 195.105 56.0717 193.529 55.355C191.952 54.6383 190.734 53.6207 189.874 52.302C189.042 50.9547 188.612 49.378 188.584 47.572H193.959C193.959 48.862 194.417 49.8797 195.335 50.625C196.281 51.3417 197.556 51.7 199.162 51.7C200.71 51.7 201.914 51.3417 202.774 50.625C203.662 49.9083 204.107 48.9193 204.107 47.658C204.107 46.5973 203.791 45.68 203.161 44.906C202.559 44.1033 201.684 43.5587 200.538 43.272L196.926 42.283C194.46 41.6523 192.554 40.52 191.207 38.886C189.888 37.252 189.229 35.274 189.229 32.952C189.229 31.1747 189.63 29.6267 190.433 28.308C191.235 26.9893 192.368 25.9717 193.83 25.255C195.32 24.5383 197.069 24.18 199.076 24.18C202.114 24.18 204.522 24.9683 206.3 26.545C208.077 28.093 208.98 30.1857 209.009 32.823H203.634C203.634 31.5903 203.232 30.63 202.43 29.942C201.627 29.2253 200.495 28.867 199.033 28.867C197.628 28.867 196.539 29.1967 195.765 29.856C194.991 30.5153 194.604 31.447 194.604 32.651C194.604 33.7403 194.89 34.672 195.464 35.446C196.066 36.1913 196.926 36.7217 198.044 37.037L201.785 38.069C204.279 38.6997 206.185 39.832 207.504 41.466C208.822 43.0713 209.482 45.0637 209.482 47.443C209.482 49.2203 209.052 50.797 208.192 52.173C207.332 53.5203 206.128 54.5667 204.58 55.312C203.032 56.0573 201.211 56.43 199.119 56.43ZM228.396 56C226.131 56 224.354 55.3693 223.064 54.108C221.803 52.8467 221.172 51.1123 221.172 48.905V37.209H214.765V32.35H221.172V25.685H226.59V32.35H235.663V37.209H226.59V48.905C226.59 50.3957 227.321 51.141 228.783 51.141H235.233V56H228.396ZM252.599 56.43C249.589 56.43 247.224 55.5987 245.504 53.936C243.784 52.2447 242.924 49.9513 242.924 47.056V32.35H248.299V47.013C248.299 48.5323 248.672 49.7077 249.417 50.539C250.163 51.3417 251.223 51.743 252.599 51.743C253.947 51.743 254.993 51.3417 255.738 50.539C256.512 49.7077 256.899 48.5323 256.899 47.013V32.35H262.274V47.056C262.274 49.9513 261.4 52.2447 259.651 53.936C257.903 55.5987 255.552 56.43 252.599 56.43ZM277.361 56.43C275.011 56.43 273.104 55.613 271.642 53.979C270.209 52.345 269.492 50.152 269.492 47.4V40.993C269.492 38.2123 270.209 36.005 271.642 34.371C273.076 32.737 274.982 31.92 277.361 31.92C279.311 31.92 280.859 32.479 282.005 33.597C283.152 34.6863 283.725 36.1913 283.725 38.112L282.521 36.865H283.768L283.596 31.232V24.61H288.971V56H283.725V51.485H282.521L283.725 50.238C283.725 52.1587 283.152 53.678 282.005 54.796C280.859 55.8853 279.311 56.43 277.361 56.43ZM279.253 51.786C280.629 51.786 281.69 51.3847 282.435 50.582C283.209 49.7507 283.596 48.604 283.596 47.142V41.208C283.596 39.746 283.209 38.6137 282.435 37.811C281.69 36.9797 280.629 36.564 279.253 36.564C277.877 36.564 276.802 36.9653 276.028 37.768C275.254 38.5707 274.867 39.7173 274.867 41.208V47.142C274.867 48.6327 275.254 49.7793 276.028 50.582C276.802 51.3847 277.877 51.786 279.253 51.786ZM296.534 56V51.098H304.79V37.209H297.609V32.35H309.95V51.098H317.26V56H296.534ZM306.94 28.394C305.85 28.394 304.99 28.1217 304.36 27.577C303.729 27.0037 303.414 26.244 303.414 25.298C303.414 24.352 303.729 23.6067 304.36 23.062C304.99 22.4887 305.85 22.202 306.94 22.202C308.029 22.202 308.889 22.4887 309.52 23.062C310.15 23.6067 310.466 24.352 310.466 25.298C310.466 26.244 310.15 27.0037 309.52 27.577C308.889 28.1217 308.029 28.394 306.94 28.394ZM332.949 56.387C330.942 56.387 329.193 56.0143 327.703 55.269C326.241 54.495 325.094 53.42 324.263 52.044C323.46 50.6393 323.059 48.991 323.059 47.099V41.251C323.059 39.359 323.46 37.725 324.263 36.349C325.094 34.9443 326.241 33.8693 327.703 33.124C329.193 32.35 330.942 31.963 332.949 31.963C334.984 31.963 336.733 32.35 338.195 33.124C339.657 33.8693 340.789 34.9443 341.592 36.349C342.423 37.725 342.839 39.3447 342.839 41.208V47.099C342.839 48.991 342.423 50.6393 341.592 52.044C340.789 53.42 339.657 54.495 338.195 55.269C336.733 56.0143 334.984 56.387 332.949 56.387ZM332.949 51.7C334.382 51.7 335.486 51.313 336.26 50.539C337.062 49.7363 337.464 48.5897 337.464 47.099V41.251C337.464 39.7317 337.062 38.585 336.26 37.811C335.486 37.037 334.382 36.65 332.949 36.65C331.544 36.65 330.44 37.037 329.638 37.811C328.835 38.585 328.434 39.7317 328.434 41.251V47.099C328.434 48.5897 328.835 49.7363 329.638 50.539C330.44 51.313 331.544 51.7 332.949 51.7Z"
        fill="currentColor"
      />
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
          <Link href="/" style={{ color: "var(--ss-primary)" }}>
            <Logo />
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
            <Link href="https://github.com/frectonz/sql-studio/releases/tag/0.1.51" className="ld-up mb-6 inline-block border px-3 py-1 text-xs font-bold tracking-widest" style={{ animationDelay: "0.1s", color: "var(--ss-primary)", borderColor: "var(--ss-primary)" }} target="_blank" rel="noopener noreferrer">
              v0.1.51
            </Link>

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
          <Link href="/" style={{ color: "var(--ss-primary)" }}>
            <Logo width={100} />
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
