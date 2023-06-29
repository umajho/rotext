export type TextNode = string;
export type InlineNode = InlineElement | TextNode;
export type BlockNode = BlockElement;

export type Properties = Record<string, unknown>;

export type RawTextSlot = string;
export type InlineSlot = InlineNode[];
export type BlockSlot = BlockNode[];
export type BlockOrInlineSlot = BlockSlot | InlineSlot;

export type RootElement = { type: "root"; slot: BlockSlot };

export type InlineElement =
  | { type: "br" }
  | { type: "em" | "em.strong" | "em.mark" | "u" | "s"; slot: InlineSlot }
  | { type: "ruby"; slots: { base: InlineSlot; text: InlineSlot } }
  | { type: "code"; slot: RawTextSlot }
  | { type: "ref-link"; slot: RawTextSlot }
  | { type: "dicexp"; slots: { code: RawTextSlot; assignTo?: RawTextSlot } };

export type BlockElement =
  | { type: "P"; slot: InlineSlot }
  | { type: "THEMATIC-BREAK" }
  | { type: "H"; props: { level: 1 | 2 | 3 | 4 | 5 | 6 }; slot: InlineSlot }
  | { type: "QUOTE"; slot: BlockOrInlineSlot }
  | { type: "OL" | "UL"; items: BlockOrInlineSlot[] }
  | { type: "DL"; items: DescriptionListItem[] }
  | { type: "TABLE"; cells: TableCell[][] };

export interface DescriptionListItem {
  type: "DL:T" | "DL:D";
  slot: BlockOrInlineSlot;
}
export interface TableCell {
  type: "TABLE:H" | "TABLE:D";
  slot: BlockOrInlineSlot;
}

export function root(slot: BlockSlot): RootElement {
  return { type: "root", slot };
}

export function text(text: string): TextNode {
  return text;
}

export function inlineBreak(): InlineElement {
  return { type: "br" };
}

export function inlineDecoration(
  type: "em" | "em.strong" | "em.mark" | "u" | "s",
  slot: InlineSlot,
): InlineElement {
  return { type, slot };
}

export function inlineRuby(base: InlineSlot, text: InlineSlot): InlineElement {
  return { type: "ruby", slots: { base, text } };
}

export function inlineCode(slot: RawTextSlot): InlineElement {
  return { type: "code", slot };
}

export function inlineRefLink(slot: RawTextSlot): InlineElement {
  return { type: "ref-link", slot };
}

export function inlineDicexp(
  code: RawTextSlot,
  assignTo?: RawTextSlot,
): InlineElement {
  return { type: "dicexp", slots: { code, assignTo } };
}

export function blockParagraph(slot: InlineSlot): BlockElement {
  return { type: "P", slot };
}

export function blockThematicBreak(): BlockElement {
  return { type: "THEMATIC-BREAK" };
}

export function blockHeading(
  level: 1 | 2 | 3 | 4 | 5 | 6,
  slot: InlineSlot,
): BlockElement {
  return { type: "H", props: { level }, slot };
}

export function blockQuote(slot: BlockOrInlineSlot): BlockElement {
  return { type: "QUOTE", slot };
}

export function blockList(
  type: "OL" | "UL",
  items: BlockOrInlineSlot[],
): BlockElement {
  return { type, items };
}

export function blockDescriptionList(
  items: DescriptionListItem[],
): BlockElement {
  return { type: "DL", items };
}

export function blockTable(cells: TableCell[][]): BlockElement {
  return { type: "TABLE", cells };
}
