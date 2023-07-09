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

//==== INLINE LEVEL ====//

Line
  = _ inlines:Inline+
    { return joinInlines(inlines); }

Inline
  = Escape
  / Reference
  / Code
  / Bold
  / Italic
  / Underline
  / Strikethrough
  / Emphasis
  / Ruby
  / [^\r\n]

Escape
  = "\\" @.

Reference
  = ">>" id:$RID
    { return create.ref_link(id); }

/**
  引用 ID 分为两类，一类是纯数字，另一类是由多级逐一限定范围而成。
  注：为方便阅读，示例中会添加 `>>` 组成完整的引用。示例中小括号代表可选。

  纯数字的 ID 被称为「数字帖号」，只能用于引用帖子（而非串），必须带有「全局前缀」。
  例：`>>TP.1234`

  多级的 ID 由「前缀」「串号」「楼层号」「子级串部分」这些部分组成：
  - 前缀：
    如果其余部分都是绝对形式（即整体指向唯一的事物），则可以拥有「全局前缀」（也可以省略）；
    否则，前缀必须为空（不带前缀）。
    由任意多的大写字母，以及位于最后的 “.” 组成。例：“TP.”。
  - 串号：
    代表串的 ID。单独存在时，用于指代串本身。
    - 绝对形式：由任意多的小写字母组成。比如：`>>(TP.)abc`。
    - 相对形式：
      - 在空缺时（比如只有楼层号时），代表当前串。比如：`>>#42`。
      - 在为 `~` 时，代表主串（相对于子串或主串本身而言）。比如：`>>~#42`。
  - 楼层号：
    代表帖在串中所处的楼层。与「子级串部分」互斥。只存在绝对形式。
    以 “#” 开头，其后由正整数组成。比如：`>>#42`、`>>(TP.)abc#42`。
  - 子级串部分：
    代表子级串或子级串中的帖子。与「楼层号」互斥。只存在绝对形式。
    以 `/` 开头，其后由「子级串的串号」和可选的「子级串的楼层号」组成。
    比如：`>>~/abc#123`、`>>(TP.)abc/abc`。
*/
RID
  = RID_GlobalPrefix ( RID_PostNumber / RID_Global )
  / RID_Relative
  / RID_Global

RID_PostNumber = [0-9]+
RID_GlobalPrefix = [A-Z]+ "."
RID_Global
  = RID_ThreadID ( RID_FloorNumber / RID_SubPart )?
RID_Relative
  = RID_FloorNumber
  / RID_ThreadRelative ( RID_FloorNumber / RID_SubPart )?

RID_ThreadID = [a-z]+
RID_ThreadRelative = "~"
RID_FloorNumber = "#" [0-9]+
RID_SubPart = "/" RID_ThreadID RID_FloorNumber?

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

Ruby
  = "[" base:(![(（] @Inline)* pLeft:[(（] text:(!([)）] "]") @Inline)* pRight:[)）] "]"
    {
      return create.ruby(
        joinInlines(base),
        [pLeft, pRight],
        joinInlines(text),
      );
    }

_ = [ \t]*
__ = [ \t\r\n]*
br = "\r\n" / "\r" / "\n"
