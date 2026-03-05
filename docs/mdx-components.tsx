import { getMDXComponents } from "@farming-labs/theme/mdx";
import type { MDXComponents } from "mdx/types";

export function useMDXComponents(components?: MDXComponents): MDXComponents {
  return getMDXComponents(components);
}
