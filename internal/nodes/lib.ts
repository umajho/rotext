export type TextNode = string;
export type InlineNode = InlineElement | TextNode;
export type BlockNode = BlockElement;

export type Properties = Record<string, unknown>;

export type RawTextSlot = string;
export type InlineSlot = InlineNode[];
export type BlockSlot = BlockNode[];
export type MixedSlot = (InlineNode | BlockNode)[];
export type BlockOrInlineSlot = BlockSlot | InlineSlot;

export type RootElement = { type: "root"; slot: BlockSlot };

export type InlineElement =
  | { type: "br" }
  | { type: "em" | "em.strong" | "em.dotted" | "u" | "s"; slot: InlineSlot }
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

export type BlockElement =
  | { type: "P"; slot: InlineSlot }
  | { type: "THEMATIC-BREAK" }
  | { type: "H"; props: { level: 1 | 2 | 3 | 4 | 5 | 6 }; slot: InlineSlot }
  | { type: "QUOTE"; slot: BlockOrInlineSlot }
  | { type: "OL" | "UL"; items: BlockOrInlineSlot[] }
  | { type: "DL"; items: DescriptionListItem[] }
  | { type: "TABLE"; slots?: { caption?: InlineSlot }; cells: TableCell[][] };

export interface DescriptionListItem {
  type: "DL:T" | "DL:D"; // `<dt/>` | `<dd/>`
  slot: BlockOrInlineSlot;
}
export interface TableCell {
  type: "TABLE:H" | "TABLE:D"; // `<th/>` | `<td/>`
  slot: MixedSlot;
}

/**
 * 提供用于创建节点的函数。
 * 其中小写者为行内元素，大写者为块级元素。
 */
export const create = {
  /** 最外部的元素 */
  ROOT(slot: BlockSlot): RootElement {
    return { type: "root", slot };
  },

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
    subType: null | "strong" | "dotted",
    slot: InlineSlot,
  ): InlineElement & { type: "em" | "em.strong" | "em.dotted" } {
    if (subType) {
      return { type: `em.${subType}`, slot };
    }
    return { type: "em", slot };
  },

  /** 下划线 */
  u(slot: InlineSlot): InlineElement & { type: "u" } {
    return { type: "u", slot };
  },

  /** 删除线 */
  s(slot: InlineSlot): InlineElement & { type: "s" } {
    return { type: "s", slot };
  },

  /** 为行内元素添加 ruby 文字，即旁注 */
  ruby(
    base: InlineSlot,
    p: [left: string, right: string],
    text: InlineSlot,
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

  /** 有序列表 */
  OL(items: BlockOrInlineSlot[]): BlockElement & { type: "OL" } {
    return { type: "OL", items };
  },

  /** 无序列表 */
  UL(items: BlockOrInlineSlot[]): BlockElement & { type: "UL" } {
    return { type: "UL", items };
  },

  /** 描述列表 */
  DL(items: DescriptionListItem[]): BlockElement & { type: "DL" } {
    return { type: "DL", items };
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
  ): TableCell & { type: "TABLE:H" | "TABLE:D" } {
    return { type: `TABLE:${type}`, slot };
  },
};
