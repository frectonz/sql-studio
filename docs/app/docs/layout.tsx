import { createDocsLayout, createDocsMetadata } from "@farming-labs/theme";
import type { Metadata } from "next";
import docsConfig from "@/docs.config";

export const metadata: Metadata = {
  ...createDocsMetadata(docsConfig),
};

export default createDocsLayout(docsConfig);
