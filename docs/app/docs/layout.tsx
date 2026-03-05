import type { Metadata } from "next";
import docsConfig from "@/docs.config";
import { createDocsLayout, createDocsMetadata } from "@farming-labs/theme";

export const metadata: Metadata = {
  ...createDocsMetadata(docsConfig),
};

export default createDocsLayout(docsConfig);
