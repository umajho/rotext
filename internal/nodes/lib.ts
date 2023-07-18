export type TextNode = string;
export type Element = BlockElement | InlineElement;
export type InlineNode = InlineElement | TextNode;
export type BlockNode = BlockElement;

export type Properties = Record<string, unknown>;

export type RawTextSlot = string;
export type InlineSlot = InlineNode[];
export type BlockSlot = BlockNode[];
export type MixedSlot = (InlineNode | BlockNode)[];
export type BlockOrInlineSlot = BlockSlot | InlineSlot;

export type Document = {
  slot: BlockSlot;
  metadata?: Record<string, any>;
};

/**
 * 所有字母必须小写，并且需以小写字母打头。
 */
export type InlineElement =
  | { type: "br" }
  | { type: "em.strong" | "s"; slot: InlineSlot }
  | {
    type: "ruby";
    props: {
      // fallback parenthesis
      p: [left: string, right: string];
    };
    slots: { base: InlineSlot; text: InlineSlot };
  }
  | { type: "code"; slot: RawTextSlot }
  | { type: "ref-link"; slot: RawTextSlot }
  | { type: "dicexp"; slots: { code: RawTextSlot; assignTo?: RawTextSlot } };

/**
 * 所有字母必须大写，并且需以大写字母打头。
 */
export type BlockElement =
  | { type: "P"; slot: InlineSlot }
  | { type: "THEMATIC-BREAK" }
  | { type: "H"; props: { level: 1 | 2 | 3 | 4 | 5 | 6 }; slot: InlineSlot }
  | { type: "QUOTE"; slot: MixedSlot }
  | { type: "OL" | "UL"; items: ListItem[] }
  | { type: "DL"; items: DescriptionListItem[] }
  | { type: "TABLE"; slots?: { caption?: InlineSlot }; cells: TableCell[][] };

export interface ListItem {
  slot: MixedSlot;
}
export interface DescriptionListItem {
  type: "DL:T" | "DL:D"; // `<dt/>` | `<dd/>`
  slot: MixedSlot;
}
export interface TableCell {
  type: "TABLE:H" | "TABLE:D"; // `<th/>` | `<td/>`
  slot: MixedSlot;
}

/** 创建文档 */
export function createDocument(
  slot: BlockSlot,
  metadata?: Record<string, any>,
) {
  return { slot, metadata };
}

/**
 * 提供用于创建节点的函数。
 * 其中小写者为行内元素，大写者为块级元素。
 *
 * NOTE: 文档不属于节点，因此使用单独的 `createDocument` 函数来创建。
 */
export const create = {
  /**
   * ~~纯粹的文本（节点）。~~
   * 直接使用 string。
   */
  // text(text: string): TextNode {
  //   return text;
  // },

  /** 行内元素间的换行 */
  br(): InlineElement & { type: "br" } {
    return { type: "br" };
  },

  /**
   * 使行内元素表达强调。
   * `strong` 代表加粗；`dotted` 代表着重号；默认是正常的 `em`，一般体现为斜体 */
  em(
    subType: "strong",
    slot: InlineSlot,
  ): InlineElement & { type: "em.strong" } {
    return { type: `em.${subType}`, slot };
  },

  /** 删除线 */
  s(slot: InlineSlot): InlineElement & { type: "s" } {
    return { type: "s", slot };
  },

  /** 为行内元素添加 ruby 文字，即注音 */
  ruby(
    base: InlineSlot,
    text: InlineSlot,
    p: [left: string, right: string],
  ): InlineElement & { type: "ruby" } {
    return { type: "ruby", props: { p }, slots: { base, text } };
  },

  /** 行内显示代码片段 */
  code(slot: RawTextSlot): InlineElement & { type: "code" } {
    return { type: "code", slot };
  },

  /** 引用链接 */
  ref_link(slot: RawTextSlot): InlineElement & { type: "ref-link" } {
    return { type: "ref-link", slot };
  },

  /** 除了记录 dicexp 之外，还记录可能的赋值对象的信息 */
  dicexp(
    code: RawTextSlot,
    assignTo?: RawTextSlot,
  ): InlineElement & { type: "dicexp" } {
    return { type: "dicexp", slots: { code, assignTo } };
  },

  /** 段落 */
  P(slot: InlineSlot): BlockElement & { type: "P" } {
    return { type: "P", slot };
  },

  /** 用于切断前后文主题，一般体现为分隔符 */
  THEMATIC_BREAK(): BlockElement & { type: "THEMATIC-BREAK" } {
    return { type: "THEMATIC-BREAK" };
  },

  /** 标题（Heading） */
  H(
    level: 1 | 2 | 3 | 4 | 5 | 6,
    slot: InlineSlot,
  ): BlockElement & { type: "H" } {
    return { type: "H", props: { level }, slot };
  },

  /** 块引用，由于 “块级” 已经体现在大写上，去掉一般名字中的 “blcok” */
  QUOTE(slot: BlockOrInlineSlot): BlockElement & { type: "QUOTE" } {
    return { type: "QUOTE", slot };
  },

  /** 有序列表及无序列表 */
  LIST(
    type: "O" | "U",
    items: ListItem[],
  ): BlockElement & { type: "OL" | "UL" } {
    return { type: `${type}L`, items };
  },

  LIST$item(slot: MixedSlot): ListItem {
    return { slot };
  },

  /** 描述列表 */
  DL(items: DescriptionListItem[]): BlockElement & { type: "DL" } {
    return { type: "DL", items };
  },

  DL$item(type: "T" | "D", slot: MixedSlot): DescriptionListItem {
    return { type: `DL:${type}`, slot };
  },

  /** 表格 */
  TABLE(
    caption: InlineSlot | null,
    cells: TableCell[][],
  ): BlockElement & { type: "TABLE" } {
    return {
      type: "TABLE",
      ...(caption ? { slots: { caption } } : {}),
      cells,
    };
  },

  TABLE$cell(
    type: "H" | "D",
    slot: MixedSlot,
  ): TableCell {
    return { type: `TABLE:${type}`, slot };
  },
};
