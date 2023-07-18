{{
  import { create, createDocument } from "@rotext/nodes";
  import { joinInlines, joinLines } from "./line.ts"
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

Block
  = ThematicBreak
  / Heading
  / Containers
  / Table
  / Paragraph

BlockLineBegin
  = [-=>#*;:] / "{|" / "|" [}+-]?

Paragraph
  = first:Line rest:(br !BlockLineBegin @Line)*
    { return recording(create.P(joinLines([first, ...rest], breaks))); }

ThematicBreak
  = "-"|3..| _
    { return recording(create.THEMATIC_BREAK()); }

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

      return recording(create.H(realLevels, line));
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
*/
Containers // TODO: `; foo : bar`
  = first:ContainersLine rest:(br @ContainersLine)*
    { return buildContainers([first, ...rest], breaks, locMap); }
ContainersLine
  = prefix:$[>#*;:]|1..| ([ \t]+ / &br) line:Line?
    { 
      return locMap
        ? { prefix, line, location: location() }
        : { prefix, line };
    }

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
  = TableCaptionBegin inlines:Inline*
    { return joinInlines(inlines); }
TableCaptionBegin = !"|}" "|+" _

TableRow
  = cells:TableRowFragment*
    { return recording(cells.flat()); }
TableRowFragment
  = br __ !TableRelatedLineBeginNoCells
    initCells:(@TableInlineCell ("!" &"!" / "|" & "|"))*
    lastCell:TableLastCell
    _
    { return [...initCells, ...(lastCell ? [lastCell] : [])] }

TableInlineCell
  = symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)*
  { return create.TABLE$cell(symbol == "!" ? "H" : "D", joinInlines(inlines)); }
TableLastCell
  = symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)* __
    !TableRelatedLineBegin first:Block?
    rest:(br !TableRelatedLineBegin @Block)*
    {
      inlines = joinInlines(inlines)
      first = first ? [...inlines, first] : inlines;
      return create.TABLE$cell(symbol == "!" ? "H" : "D", [...first, ...rest]);
    }
  / symbol:TableCellBegin
    inlines:(!TableCellBegin @Inline)* _
    { return create.TABLE$cell(symbol == "!" ? "H" : "D", joinInlines(inlines)); }
TableCellBegin = !TableRelatedLineBeginNoCells @TableCellLineBegin

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
  = "``" text:$(!"``" @[^\r\n])* "``"
    { return create.code(text); }
  / "[`" text:$(!"`]" @[^\r\n])* "`]"
    { return create.code(text); }

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
