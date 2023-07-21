{{
  import { create, createDocument } from "@rotext/nodes";
  import { joinInlines, joinLines, trimInlinesEndShallowInPlace } from "./line.ts";
  import { buildContainers } from "./container.ts";
}}

{
  const breaks = options.breaks;
  const locMap = options.recordsLocation ? new WeakMap() : null;

  function recording(el) {
    locMap?.set(el, location());
    return el;
  }
}

Document
  = __ unflattenBlocks:(@Block (br __ / !.))+
    { 
      const blocks = unflattenBlocks.flat(); // 用于展平 `Containers`
      const metadata = locMap ? { locMap } : undefined;
      return createDocument(blocks, metadata);
    } 
  / __
    { return createDocument([]); }

Block = BlockWithoutParagraph / Paragraph

BlockInTable = BlockWithoutParagraph / ParagraphInTable

BlockWithoutParagraph
  = ThematicBreak
  / Heading
  / Containers
  / Table

Paragraph
  = first:Line rest:(br @Line)*
    { return recording(create.P(joinLines([first, ...rest], breaks))); }

ParagraphInTable
  = first:Line rest:(br !TableRelatedLineBegin @Line)*
    { return recording(create.P(joinLines([first, ...rest], breaks))); }

ThematicBreak
  = "-"|3..| _
    { return recording(create.THEMATIC_BREAK()); }

Heading
  = level:$"="|1..6| spaces:$[ \t]+
    line:Line
    &{ // 末尾至少需要有一个对应的标记符号
      let last = line[line.length - 1];
      if (typeof last !== "string") return false;
      return last.trimEnd().endsWith(" " + level);
    }
    {
      const last = line[line.length - 1]
        .trimEnd()
        .slice(0, line[line.length - 1].length - level.length)
        .trimEnd();

      if (last === "") {
        line.pop()
      } else {
        line[line.length - 1] = last;
      }

      return recording(create.H(level.length, line));
    }

// NOTE: 之所以是复数，是因为可能会有得到复数个容器的情况
/**
  按照前缀可以将容器分为以下几组：
  - `>`：引言；
  - `#`：有序列表；
  - `*`：无序列表；
  - `;` 和 `:`：描述列表。
  同组的行上下挨在一起时，形成一个整体，而其中的每一行成为一项。
  此外，可以在已有的容器项之下用 `>` 来延长那个容器项，以实现一个容器项占据多行。

  除空行外，容器的前缀与内容之间需要有空白。嵌套时，内、外层容器的前缀之间也需要有空白。
*/
Containers // TODO: `; foo : bar`
  = first:ContainersLine rest:(br @ContainersLine)*
    { return buildContainers([first, ...rest], breaks, locMap); }
ContainersLine
  = prefix:ContainerPrefix+ line:Line?
    { 
      console.log()
      return locMap
        ? { prefix: prefix.join(""), line, location: location() }
        : { prefix: prefix.join(""), line };
    }
ContainerPrefix = @[>#*;:] ([ \t]+ / &br)

Table
  = TableBegin /*无视首行后边的内容*/ @TableRows __ TableEnd
TableBegin = "{|" (!("|}") [^\r\n])*
TableEnd = "|}" _ &(br / !.) // NOTE: 为了实现简单，表格后不能有其他内容

TableRelatedLineBeginNoCells = "|" [}+-]
TableCellLineBegin = [!|]
TableRelatedLineBegin = (TableRelatedLineBeginNoCells / TableCellLineBegin)

TableRows
    = caption:(br __ @TableCaption)? first:TableRow? rest:(br __ TableRowBegin @TableRow)*
      { 
        const table = create.TABLE(
          caption ? caption : null,
          [...(first?.length ? [first] : []), ...(rest ?? [])],
        );
        return recording(table);
      }
TableRowBegin = !"|}" "|" "-"+ _

// NODE: Mediawiki 的 caption 可以在行中，可以是块上下文。
//       这里为了实现简单就只允许在第二行，且只允许行内元素了。
TableCaption
  = TableCaptionBegin [ \t]+ inlines:Inline*
    { return joinInlines(inlines); }
TableCaptionBegin = !"|}" "|+"

TableRow
  = cells:TableRowFragment*
    { return recording(cells.flat()); }
TableRowFragment
  = br __ !TableRelatedLineBeginNoCells
    initCells:(@TableInlineCell ("!" &"!" / "|" & "|"))*
    &{
      for (let i = 0; i < initCells.length - 1; i++) {
        const cell = initCells[i];
        if (!cell.__hasSpacesAtEnd) {
          return false;
        }
        delete cell.__hasSpacesAtEnd;
      }
      if (initCells.length) {
        delete initCells[initCells.length - 1].__hasSpacesAtEnd;
      }
      return true;
    }
    lastCell:TableLastCell
    _
    { return [...initCells, ...(lastCell ? [lastCell] : [])] }

TableInlineCell
  = symbol:TableCellBegin ([ \t]+ / &br)
    inlines:(!TableCellBegin @Inline)*
  {
    let hasSpacesAtEnd;
    if (inlines.length) {
      inlines = joinInlines(inlines);

      [inlines, hasSpacesAtEnd] = trimInlinesEndShallowInPlace(inlines);
    }

    const cell = create.TABLE$cell(symbol == "!" ? "H" : "D", inlines ?? []);
    cell.__hasSpacesAtEnd = hasSpacesAtEnd;
    return cell;
  }
TableLastCell
  = symbol:TableCellBegin ([ \t]+ / &br)
    inlines:Inline* _
    blocks:BlocksInTableCell?
    {
      if (inlines.length) {
        inlines = joinInlines(inlines);

        let _;
        [inlines, _] = trimInlinesEndShallowInPlace(inlines);
      }

      let slot = inlines;
      if (blocks?.length) {
        slot = slot ? [...slot, ...blocks] : blocks;
      }
      return create.TABLE$cell(symbol == "!" ? "H" : "D", slot);
    }
TableCellBegin = !TableRelatedLineBeginNoCells @TableCellLineBegin

BlocksInTableCell
  = !TableRelatedLineBegin first:BlockInTable?
    rest:(br __ !TableRelatedLineBegin @BlockInTable)*
    { return first ? [first, ...rest.flat()] : rest.flat(); }

//==== INLINE LEVEL ====//

Line
  = _ inlines:Inline+
    { return joinInlines(inlines); }

Inline
  = Escape
  / Reference
  / Code
  / Bold
  / Strikethrough
  / Ruby
  / [^\r\n]

Escape
  = "\\" @.

Reference
  = "[>>" id:$RID "]"
    { return create.ref_link(id); }

/**
  引用 ID 分为两类，一类是纯数字，另一类是由多级逐一限定范围而成。
  注·1：为方便阅读，示例中会包上 `[>>…]` 组成完整的引用。示例中小括号代表可选。
  注·2：小括号（`(…)`）代表内容可选。

  纯数字的 ID 被称为「数字帖号」，只能用于引用帖子（而非串），必须带有「全局前缀」。
  例：`[>>TP.1234]`

  多级的 ID 由「全局前缀」「串号」「楼层号」「子级串部分」这些部分组成：
  - 全局前缀：
    如果其余部分都是绝对形式（即整体指向唯一的事物），则可以拥有「全局前缀」（也可以省略）；
    否则，前缀必须为空（不带前缀）。
    由任意多的大写字母，以及位于最后的 “.” 组成。例：“TP.”。
  - 串号：
    代表串的 ID。单独存在时，用于指代串本身。
    - 绝对形式：由任意多的小写字母组成。比如：`[>>(TP.)abc]`。
    - 相对形式：
      - 在空缺时（比如只有楼层号时），代表当前串。比如：`[>>#42]`。
      - 在为 `~` 时，代表主串（相对于子串或主串本身而言）。比如：`[>>~#42]`。
  - 楼层号：
    代表帖在串中所处的楼层。与「子级串部分」互斥。
    绝对形式：以 “#” 开头，其后由正整数组成。比如：`[>>#42]`、`[>>(TP.)abc#42]`。
    相对形式：`[>>#]`，代表当前楼层。
  - 子级串部分：
    代表子级串或子级串中的帖子。与「楼层号」互斥。只存在绝对形式。
    以 `/` 开头，其后由「子级串的串号」和可选的「子级串的楼层号」组成。
    比如：`[>>~/abc#123]`、`[>>(TP.)abc/def]`。
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
  = RID_FloorNumberMaybeRelative
  / RID_ThreadRelative ( RID_FloorNumber / RID_SubPart )?

RID_ThreadID = [a-z]+
RID_ThreadRelative = "~"
RID_FloorNumber = "#" [0-9]+
RID_FloorNumberMaybeRelative = "#" [0-9]*
RID_SubPart = "/" RID_ThreadID RID_FloorNumber?

Code
  = "[`" text:(!"`]" @CodeText)* "`]"
    { return create.code(text.join("")); }

CodeText
  = @"`" "`"
  / $[^`]+

Bold
  = "['" inlines:(!"']" @Inline)* "']"
    { return create.em("strong", joinInlines(inlines)); }

Strikethrough
  = "[~" inlines:(!"~]" @Inline)* "~]"
    { return create.s(joinInlines(inlines)); }

Ruby
  = "[" base:(![(（] @Inline)* pLeft:[(（] text:(!([)）] "]") @Inline)* pRight:[)）] "]"
    {
      return create.ruby(
        joinInlines(base),
        joinInlines(text),
        [pLeft, pRight],
      );
    }

_ = [ \t]*
__ = [ \t\r\n]*
br = "\r\n" / "\r" / "\n"
