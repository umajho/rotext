{{
  import { joinInlines, joinLines } from "./line.ts"
  import { buildList } from "./list.ts";

  function dummyH(el, props, children) {
    return {
      el,
      ...(props ? {props} : {}),
      ...(children ? {children} : {}),
    };
  }
  function dummyFragment(nodes) {
    return { el: null, children: nodes };
  }
}}

{
  options = options ?? {};
  const breaks = options.breaks ?? false;
  const v = options.v ?? { h: dummyH, fragment: dummyFragment };
  const h = v.h;
}

Document
  = __ blocks:(@Block (br __ / !.))+
    { return h("article", {}, blocks); }
  / __
    { return h("article", {}, []); }

Block
  = ThematicBreak
  / Heading
  / Blockquote
  / List
  / Table
  / Paragraph

BlockLineBegin
  = [-=>#*;:] / "{|" / "|" [}+-]?

Paragraph
  = first:Line rest:(br !BlockLineBegin @Line)*
    { return h("p", {}, joinLines([first, ...rest], breaks, h)) }

ThematicBreak
  = "-"|3..| _
    { return h("hr"); }

Heading
  = level:$"="|1..6|
  	line:Line
    &{
      const last = line[line.length - 1];
      if (typeof last !== "string") return false;
      return last.trimEnd().endsWith("=".repeat(level.length));
    }
    {
      const last = line[line.length - 1].trimEnd();
      line = [...line];
      line[line.length - 1] = last.slice(0, last.length - level.length);
      return h(`h${level.length}`, {}, line);
    }

Blockquote
  = first:BlockquoteLine rest:(br @BlockquoteLine)*
    { return h("blockquote", {}, joinLines([first, ...rest], breaks, h)) }
BlockquoteLine
  = ">" !">" @Line
  / ">" !">" _
    { return [""] }

List // TODO `; foo : bar`
  = first:ListItem rest:(br @ListItem)*
    { return buildList([first, ...rest], v); }

ListItem
  = level:$[#*;:]|1..| line:Line
    { return [level, line]; }

Table
  = TableBegin /*无视首行后边的内容*/ rows:TableRows __ TableEnd
    { return h("table", {}, rows); }
TableBegin = "{|" (!("|}") [^\r\n])*
TableEnd = "|}" _ &(br / !.) // NOTE: 为了实现简单，表格后不能有其他内容

TableRelatedLineBeginNoCells = "|" [}+-]
TableCellLineBegin = [!|]
TableRelatedLineBegin = (TableRelatedLineBeginNoCells / TableCellLineBegin)

TableRows
    = caption:(br __ @TableCaption)? first:TableRow? rest:(br __ TableRowBegin @TableRow)*
      { return [...(caption ? [caption] : []), ...(first ? [first] : []), ...(rest ?? [])]; }
TableRowBegin = !"|}" "|" "-"+ _

// NODE: Mediawiki 的 caption 可以在行中，可以是块上下文。
//       这里为了实现简单就只允许在第二行，且只允许行内元素了。
TableCaption
  = TableCaptionBegin inlines:Inline*
    { return h("caption", {}, inlines); }
TableCaptionBegin = !"|}" "|+" _

TableRow
  = cells:TableRowFragment*
    { return h("tr", {}, cells.flat()); }
TableRowFragment
  = br __ !TableRelatedLineBeginNoCells
    initCells:(@TableInlineCell ("!" &"!" / "|" & "|"))*
    lastCell:TableLastCell
    _
    { return [...initCells, ...(lastCell ? [lastCell] : [])] }

TableInlineCell
  = symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)*
  { return h(symbol == "!" ? "th" : "td", {}, joinInlines(inlines)); }
TableLastCell
  = symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)* __
    !TableRelatedLineBegin first:Block?
    rest:(br !TableRelatedLineBegin @Block)*
    {
      inlines = joinInlines(inlines)
      first = first ? [...inlines, first] : inlines;
      return h(symbol == "!" ? "th" : "td", {}, [...first, ...rest]);
    }
  / symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)* _
    { return h(symbol == "!" ? "th" : "td", {}, joinInlines(inlines)); }
TableCellBegin = !TableRelatedLineBeginNoCells @TableCellLineBegin

//==== INLINE LEVEL ====//

Line
  = _ inlines:Inline+
    { return joinInlines(inlines); }

Inline
  = Reference
  / Code
  / Bold
  / Italic
  / Underline
  / Strikethrough
  / Emphasis
  / Ruby
  / [^\r\n]

Reference
  = ">>" id:$ReferenceID
    { return h("span", { style: { color: "green" } }, ">>" + id); }
ReferenceID
  = [a-z]i+ "." (([a-z]i+ ("/" [a-z]i+)?) ([0-9]+)? / [0-9]+)
  / [0-9]+

Code
  = "``" inlines:(!"``" @Inline)* "``"
    { return h("code", {}, joinInlines(inlines)); }
  / "[`" inlines:(!"`]" @Inline)* "`]"
    { return h("code", {}, joinInlines(inlines)); }

Bold
  = "''" inlines:(!"''" @Inline)* "''"
    { return h("b", {}, joinInlines(inlines)); }
  / "['" inlines:(!"']" @Inline)* "']"
    { return h("b", {}, joinInlines(inlines)); }

Italic
  = "//" inlines:(!"//" @Inline)* "//"
    { return h("i", {}, joinInlines(inlines)); }
  / "[/" inlines:(!"/]" @Inline)* "/]"
    { return h("i", {}, joinInlines(inlines)); }

Underline
  = "__" inlines:(!"__" @Inline)* "__"
    { return h("u", {}, joinInlines(inlines)); }
  / "[_" inlines:(!"_]" @Inline)* "_]"
    { return h("u", {}, joinInlines(inlines)); }

Strikethrough
  = "~~" inlines:(!"~~" @Inline)* "~~"
    { return h("s", {}, joinInlines(inlines)); }
  / "[~" inlines:(!"~]" @Inline)* "~]"
    { return h("s", {}, joinInlines(inlines)); }

Emphasis
  = "[." inlines:(!".]" @Inline)* ".]"
    { return h("span.emphasis-marks", { }, joinInlines(inlines)); }

Ruby
  = "[" base:(!"(" @Inline)* lp:[(（] rt:(!([)）] "]") @Inline)* rp:[)）] "]"
    {
      base = joinInlines(base);
      const rtEl = h("rt", {}, joinInlines(rt));
      const [rpLeft, rpRight] = [h("rp", {}, lp), h("rp", {}, rp)];
      return h("ruby", {}, [...base, rpLeft, rtEl, rpRight]);
    }

_ = [ \t]*
__ = [ \t\r\n]*
br = "\r\n" / "\r" / "\n"
