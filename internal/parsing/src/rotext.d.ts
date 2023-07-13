import { Root } from "@rotext/nodes";

export interface ParseOptions {
  breaks: boolean;
}

export function parse(input: string, opts): Root;
