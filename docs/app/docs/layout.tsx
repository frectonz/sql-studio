import type { Metadata } from "next";
import docsConfig from "@/docs.config";
import { createDocsLayout, createDocsMetadata } from "@farming-labs/theme";

export const metadata: Metadata = {
  ...createDocsMetadata(docsConfig),
  openGraph: {
    type: "website",
    siteName: "SQL Studio",
    images: [{ url: "/og/docs.png", width: 1200, height: 630 }],
  },
  twitter: {
    card: "summary_large_image",
    images: ["/og/docs.png"],
  },
};

export default createDocsLayout(docsConfig);
