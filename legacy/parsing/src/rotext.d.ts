import { Document } from "@rotext/nodes";

export interface ParseOptions {
  breaks: boolean;
  recordsLocation: boolean;
}

export function parse(input: string, opts: ParseOptions): Document;
