{{
  import { create } from "@rotext/nodes";
  import { joinInlines, joinLines } from "./line.ts"
  // import { buildList } from "./list.ts";
}}

{
  options = options ?? {};
  const breaks = options.breaks ?? false;
}

Document
  = __ blocks:(@Block (br __ / !.))+
    { return create.ROOT(blocks); }
  / __
    { return create.ROOT([]); }

Block
  = ThematicBreak
  / Heading
  // / Blockquote
  // / List
  // / Table
  / Paragraph

BlockLineBegin
  = [-=>#*;:] / "{|" / "|" [}+-]?

Paragraph
  = first:Line rest:(br !BlockLineBegin @Line)*
    { return create.P(joinLines([first, ...rest], breaks)) }

ThematicBreak
  = "-"|3..| _
    { return create.THEMATIC_BREAK(); }

Heading
  = level:$"="|1..6| spaces:$_
    line:Line
    &{ // 末尾至少需要有一个对应的标记符号
      let last = line[line.length - 1];
      if (typeof last !== "string") return false;
      last = last.trimEnd();
      return last[last.length-1] === "=";
    }
    {
      let last = line[line.length - 1].trimEnd();

      // 根据右侧对应标记符号的数量确定真实的级别
      let realLevels = 1;
      for (let i = last.length - 1 - 1; i >= 0 && realLevels < level.length; i--) {
        if (last[i] === "=") {
          realLevels++;
        } else {
          break;
        }
      }

      line = [...line];

      if (realLevels < level.length) { // 返还左侧多余的标记符号
        const surplus = level.slice(realLevels) + spaces;
        if (typeof line[0] === "string") {
          line[0] = surplus + line[0];
        } else {
          line.splice(0, 0, surplus);
        }
      }

      last = line[line.length - 1];
      last = last.slice(0, last.length - realLevels).trimEnd()
      if (last === "") {
        line.pop()
      } else {
        line[line.length - 1] = last;
      }

      return create.H(realLevels, line);
    }

// Blockquote
//   = first:BlockquoteLine rest:(br @BlockquoteLine)*
//     { return h("blockquote", {}, joinLines([first, ...rest], breaks, h)) }
// BlockquoteLine
//   = ">" !">" @Line
//   / ">" !">" _
//     { return [""] }

// List // TODO `; foo : bar`
//   = first:ListItem rest:(br @ListItem)*
//     { return buildList([first, ...rest], v); }

// ListItem
//   = level:$[#*;:]|1..| line:Line
//     { return [level, line]; }

// Table
//   = TableBegin /*无视首行后边的内容*/ rows:TableRows __ TableEnd
//     { return h("table", {}, rows); }
// TableBegin = "{|" (!("|}") [^\r\n])*
// TableEnd = "|}" _ &(br / !.) // NOTE: 为了实现简单，表格后不能有其他内容

// TableRelatedLineBeginNoCells = "|" [}+-]
// TableCellLineBegin = [!|]
// TableRelatedLineBegin = (TableRelatedLineBeginNoCells / TableCellLineBegin)

// TableRows
//     = caption:(br __ @TableCaption)? first:TableRow? rest:(br __ TableRowBegin @TableRow)*
//       { return [...(caption ? [caption] : []), ...(first ? [first] : []), ...(rest ?? [])]; }
// TableRowBegin = !"|}" "|" "-"+ _

// // NODE: Mediawiki 的 caption 可以在行中，可以是块上下文。
// //       这里为了实现简单就只允许在第二行，且只允许行内元素了。
// TableCaption
//   = TableCaptionBegin inlines:Inline*
//     { return h("caption", {}, inlines); }
// TableCaptionBegin = !"|}" "|+" _

// TableRow
//   = cells:TableRowFragment*
//     { return h("tr", {}, cells.flat()); }
// TableRowFragment
//   = br __ !TableRelatedLineBeginNoCells
//     initCells:(@TableInlineCell ("!" &"!" / "|" & "|"))*
//     lastCell:TableLastCell
//     _
//     { return [...initCells, ...(lastCell ? [lastCell] : [])] }

// TableInlineCell
//   = symbol:TableCellBegin
//     inlines:(!TableCellBegin @Inline)*
//   { return h(symbol == "!" ? "th" : "td", {}, joinInlines(inlines)); }
// TableLastCell
//   = symbol:TableCellBegin
//     inlines:(!TableCellBegin @Inline)* __
//     !TableRelatedLineBegin first:Block?
//     rest:(br !TableRelatedLineBegin @Block)*
//     {
//       inlines = joinInlines(inlines)
//       first = first ? [...inlines, first] : inlines;
//       return h(symbol == "!" ? "th" : "td", {}, [...first, ...rest]);
//     }
//   / symbol:TableCellBegin
//     inlines:(!TableCellBegin @Inline)* _
//     { return h(symbol == "!" ? "th" : "td", {}, joinInlines(inlines)); }
// TableCellBegin = !TableRelatedLineBeginNoCells @TableCellLineBegin

// //==== INLINE LEVEL ====//

Line
  = _ inlines:Inline+
    { return joinInlines(inlines); }

Inline
  = Escape
//   / Reference
  / Code
  / Bold
  / Italic
  / Underline
  / Strikethrough
  / Emphasis
//   / Ruby
  / [^\r\n]

Escape
  = "\\" @.

// Reference
//   = ">>" id:$ReferenceID
//     { return h("span", { class: { "post-reference": true } }, ">>" + id); }
// /**
//   引用 ID 的组成：
//   - 「全局前缀」：
//     表示这个 ID 在站内任何地方都会指向相同的内容。
//     数字帖号必须带有此前缀（以防止与楼层号混淆），其他全局 ID 可选，非全局 ID 不可使用。
//   - 数字帖号之外的情况下，全局前缀之后的内容由三部分组成：
//     - 串号：
//       不可以省略，但可以简写为 “~”。必定构成全局 ID。
//     - 楼层号：
//       可以省略，不能与子级串部分同时出现。
//       不省略时，若含串号，构成全局 ID；否则构成局部 ID（只在同一个串内指向相同内容）。
//     - 子级串部分：
//       可以省略，由两部分组成：
//       - 子级串的串号：不可以省略。
//       - 子串的楼层号：可以省略。
// */
// ReferenceID
//   = ReferenceIDGlobalPrefix (
//     [0-9]+ // 数字帖号，用于全局引用串或帖子
//     / ReferenceIDOfGlobal
//   )
//   / ReferenceIDForFloor // 用于同串内引用帖子
//   / ReferenceIDForSub // 用于同主串内引用子串或子串中的帖子
//   / ReferenceIDOfGlobal
// ReferenceIDGlobalPrefix = [A-Z]+ "."
// ReferenceIDOfGlobal
//   = ReferenceIDForThreadFloor // 用于全局引用帖子
//   / ReferenceIDForThread ReferenceIDForSub? // 用于全局引用串，或子串，或子串中的帖子
// // 串号
// ReferenceIDForThread = [a-z]+ / "~"
// // 不含串号的楼层号
// ReferenceIDForFloor = "#" [0-9]+
// // 含串号的楼层号（楼层帖号）
// ReferenceIDForThreadFloor = ReferenceIDForThread ReferenceIDForFloor
// // 子串的含串号的楼层号（楼层帖号），或者子串的串号
// ReferenceIDForSub = "/" (ReferenceIDForThreadFloor / ReferenceIDForThread)

Code
  = "``" text:$(!"``" @[^\r\n])* "``"
    { return create.code(text); }
  / "[`" text:$(!"`]" @[^\r\n])* "`]"
    { return create.code(text); }

Bold
  = "''" inlines:(!"''" @Inline)* "''"
    { return create.em("strong", joinInlines(inlines)); }
  / "['" inlines:(!"']" @Inline)* "']"
    { return create.em("strong", joinInlines(inlines)); }

Italic
  = "//" inlines:(!"//" @Inline)* "//"
    { return create.em(null, joinInlines(inlines)); }
  / "[/" inlines:(!"/]" @Inline)* "/]"
    { return create.em(null, joinInlines(inlines)); }

Underline
  = "__" inlines:(!"__" @Inline)* "__"
    { return create.u(joinInlines(inlines)); }
  / "[_" inlines:(!"_]" @Inline)* "_]"
    { return create.u(joinInlines(inlines)); }

Strikethrough
  = "~~" inlines:(!"~~" @Inline)* "~~"
    { return create.s(joinInlines(inlines)); }
  / "[~" inlines:(!"~]" @Inline)* "~]"
    { return create.s(joinInlines(inlines)); }

Emphasis
  = "[." inlines:(!".]" @Inline)* ".]"
    { return create.em("dotted", joinInlines(inlines)); }

// Ruby
//   = "[" base:(!"(" @Inline)* lp:[(（] rt:(!([)）] "]") @Inline)* rp:[)）] "]"
//     {
//       base = joinInlines(base);
//       const rtEl = h("rt", {}, joinInlines(rt));
//       const [rpLeft, rpRight] = [h("rp", {}, lp), h("rp", {}, rp)];
//       return h("ruby", {}, [...base, rpLeft, rtEl, rpRight]);
//     }

_ = [ \t]*
__ = [ \t\r\n]*
br = "\r\n" / "\r" / "\n"
