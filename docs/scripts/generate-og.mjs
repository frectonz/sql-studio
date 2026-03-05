import fs from "fs";
import path from "path";
import satori from "satori";
import { Resvg } from "@resvg/resvg-js";

const BG = "#071611";
const PRIMARY = "#00ff9f";
const FOREGROUND = "#f8faf9";
const MUTED = "#808c87";

const DOCS_DIR = path.resolve("app/docs");
const OUT_DIR = path.resolve("public/og");

const fontPath = path.resolve("scripts/JetBrainsMono-Bold.ttf");
const fontData = fs.readFileSync(fontPath);

const pages = [];

function scanPages(dir, slugPrefix = "") {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      scanPages(path.join(dir, entry.name), `${slugPrefix}${entry.name}/`);
    }
    if (entry.name === "page.mdx") {
      const content = fs.readFileSync(path.join(dir, entry.name), "utf-8");
      const titleMatch = content.match(/^title:\s*"?([^"\n]+)"?/m);
      const descMatch = content.match(/^description:\s*"?([^"\n]+)"?/m);
      const slug = slugPrefix.replace(/\/$/, "") || "docs";
      pages.push({
        slug,
        title: titleMatch ? titleMatch[1].trim() : slug,
        description: descMatch ? descMatch[1].trim() : "",
      });
    }
  }
}

scanPages(DOCS_DIR);

pages.push({
  slug: "index",
  title: "SQL Studio",
  description:
    "Single binary SQL database explorer. 10 databases. One interface.",
});

fs.mkdirSync(OUT_DIR, { recursive: true });

function buildMarkup(page) {
  const titleSize = page.title.length > 20 ? 56 : 72;

  const children = [
    {
      type: "div",
      props: {
        style: {
          fontSize: titleSize,
          fontWeight: 700,
          color: FOREGROUND,
          letterSpacing: "-0.04em",
          lineHeight: 1,
        },
        children: page.title,
      },
    },
  ];

  if (page.description) {
    children.push({
      type: "div",
      props: {
        style: {
          fontSize: 24,
          color: MUTED,
          lineHeight: 1.4,
          maxWidth: 900,
        },
        children: page.description,
      },
    });
  }

  return {
    type: "div",
    props: {
      style: {
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        backgroundColor: BG,
        padding: 60,
        fontFamily: "JetBrains Mono",
      },
      children: [
        {
          type: "div",
          props: {
            style: {
              display: "flex",
              alignItems: "center",
              gap: 12,
              marginBottom: 40,
            },
            children: [
              {
                type: "div",
                props: {
                  style: {
                    width: 32,
                    height: 32,
                    borderRadius: 4,
                    backgroundColor: PRIMARY,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    color: BG,
                    fontSize: 14,
                    fontWeight: 700,
                  },
                  children: "SQL",
                },
              },
              {
                type: "div",
                props: {
                  style: {
                    color: PRIMARY,
                    fontSize: 20,
                    fontWeight: 700,
                    letterSpacing: "-0.02em",
                  },
                  children: "SQL STUDIO",
                },
              },
            ],
          },
        },
        {
          type: "div",
          props: {
            style: {
              display: "flex",
              flexDirection: "column",
              gap: 16,
              flex: 1,
            },
            children,
          },
        },
        {
          type: "div",
          props: {
            style: {
              height: 4,
              backgroundColor: PRIMARY,
              borderRadius: 2,
              marginTop: "auto",
            },
            children: "",
          },
        },
      ],
    },
  };
}

for (const page of pages) {
  const markup = buildMarkup(page);

  const svg = await satori(markup, {
    width: 1200,
    height: 630,
    fonts: [
      {
        name: "JetBrains Mono",
        data: fontData,
        weight: 700,
        style: "normal",
      },
    ],
  });

  const resvg = new Resvg(svg, {
    fitTo: { mode: "width", value: 1200 },
  });
  const png = resvg.render().asPng();

  const outPath = path.join(OUT_DIR, `${page.slug}.png`);
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, png);
  console.log(`Generated: ${outPath}`);
}

console.log(`\nDone! Generated ${pages.length} OG images.`);
