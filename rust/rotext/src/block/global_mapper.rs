use crate::{common::Range, global};

/// 用于将产出 [global::Event] 的流转化为便于 [Parser] 处理的流。
pub struct GlobalEventStreamMapper<'a, I: 'a + Iterator<Item = global::Event>> {
    input: &'a [u8],
    stream: I,

    deferred: Option<Mapped>,
    remain: Option<RemainUndetermined>,
    blank_at_line_beginning: Option<Range>,
}

#[derive(Debug)]
struct RemainUndetermined {
    content: Range,

    next_offset: usize,
    is_to_start: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mapped {
    /// 对应索引的字符。
    CharAt(usize),
    /// 上个字符之后（索引 + 1）的那个字符。
    NextChar,
    /// LF 换行。
    LineFeed,
    /// 空白。
    BlankAtLineBeginning(Range),
    /// 文本。
    Text(Range),
}

impl<'a, I: 'a + Iterator<Item = global::Event>> GlobalEventStreamMapper<'a, I> {
    pub fn new(input: &'a [u8], stream: I) -> GlobalEventStreamMapper<'a, I> {
        GlobalEventStreamMapper {
            input,
            stream,
            deferred: None,
            remain: None,
            blank_at_line_beginning: Some(Range::new(0, 0)),
        }
    }

    pub fn next(&mut self) -> Option<Mapped> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }

        loop {
            if let Some(ref mut remain) = self.remain {
                // 先清掉剩余的。

                if remain.next_offset == remain.content.length() {
                    // 已经没有剩余的了。
                    self.remain = None;
                    continue;
                }

                let index = remain.content.start() + remain.next_offset;
                let char = self.input[index];

                if let Some(ref mut range) = self.blank_at_line_beginning {
                    if char == b' ' {
                        remain.next_offset += 1;
                        range.set_length(range.length() + 1);
                        continue;
                    } else {
                        let range = *range;
                        self.blank_at_line_beginning = None;
                        if range.length() > 0 {
                            return Some(Mapped::BlankAtLineBeginning(range));
                        }
                    }
                }

                if char == b'\n' {
                    // 单独对待 LF，接下来剩余的字符算新开始。
                    //
                    // NOTE: CR 已经在全局阶段被无视了。

                    self.blank_at_line_beginning = Some(Range::new(index + 1, 0));
                    remain.next_offset += 1;
                    remain.is_to_start = true;
                    return Some(Mapped::LineFeed);
                } else if remain.is_to_start {
                    remain.next_offset += 1;
                    remain.is_to_start = false;
                    return Some(Mapped::CharAt(index));
                } else {
                    remain.next_offset += 1;
                    return Some(Mapped::NextChar);
                }
            }

            let Some(next) = self.stream.next() else {
                if let Some(range) = self.blank_at_line_beginning.take() {
                    if range.length() > 0 {
                        return Some(Mapped::BlankAtLineBeginning(range));
                    }
                }
                return None;
            };

            match next {
                global::Event::Undetermined(content) => {
                    self.remain = Some(RemainUndetermined {
                        content,
                        is_to_start: true,
                        next_offset: 0,
                    });
                }
                global::Event::VerbatimEscaping {
                    content,
                    is_closed_forcedly: _,
                } => {
                    let (mut start, mut length) = (content.start(), content.length());
                    if length >= 2 {
                        if self.input[start] == b' ' {
                            start += 1;
                            length -= 1;
                        }
                        if self.input[start + length - 1] == b' ' {
                            length -= 1;
                        }
                    }

                    let mapped_text = Some(Mapped::Text(Range::new(start, length)));
                    if let Some(range) = self.blank_at_line_beginning.take() {
                        if range.length() > 0 {
                            self.deferred = mapped_text;
                            return Some(Mapped::BlankAtLineBeginning(range));
                        }
                    }
                    return mapped_text;
                }
            }
        }
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for GlobalEventStreamMapper<'a, I> {
    type Item = Mapped;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use std::{time, vec};

    use super::*;
    use rstest::rstest;

    #[rstest]
    // ## 无特殊语法
    #[case("", vec![])]
    #[case("  ", vec![
        Mapped::BlankAtLineBeginning(Range::new(0, 2))])]
    #[case("a", vec![
        Mapped::CharAt(0)])]
    #[case("ab", vec![
        Mapped::CharAt(0), Mapped::NextChar])]
    // ## 换行
    #[case("a\nbc", vec![
        Mapped::CharAt(0), Mapped::LineFeed, Mapped::CharAt(2), Mapped::NextChar])]
    // ### 空行
    #[case("\n", vec![
        Mapped::LineFeed ])]
    #[case("\r\n", vec![
        Mapped::LineFeed])]
    #[case("\n\n", vec![
        Mapped::LineFeed, Mapped::LineFeed])]
    #[case("\r\n\r\n", vec![
        Mapped::LineFeed, Mapped::LineFeed])]
    #[case("a\n", vec![
        Mapped::CharAt(0), Mapped::LineFeed])]
    #[case("a\n\n", vec![
        Mapped::CharAt(0), Mapped::LineFeed, Mapped::LineFeed])]
    #[case("a\r\n\r\n", vec![
        Mapped::CharAt(0), Mapped::LineFeed, Mapped::LineFeed])]
    // #### 有空格的空行
    #[case("  \n", vec![
        Mapped::BlankAtLineBeginning(Range::new(0, 2)), Mapped::LineFeed])]
    #[case("a\n  \n", vec![
        Mapped::CharAt(0), Mapped::LineFeed,
        Mapped::BlankAtLineBeginning(Range::new(2, 2)),
        Mapped::LineFeed])]
    #[case("  <` `>\n", vec![
        Mapped::BlankAtLineBeginning(Range::new(0, 2)),
        Mapped::Text(Range::new(4, 1)), Mapped::LineFeed])]
    // ## 逐字文本转义转为文本
    #[case("<`a`>", vec![
        Mapped::Text(Range::new(2, 1))])]
    #[case("<` a `>", vec![
        Mapped::Text(Range::new(3, 1))])]
    #[case("<`  a  `>", vec![
        Mapped::Text(Range::new(3, 3))])]
    #[case("<` `>", vec![
        Mapped::Text(Range::new(2, 1))])]
    #[case("<`  `>", vec![
        Mapped::Text(Range::new(3, 0))])]
    #[case("<`   `>", vec![
        Mapped::Text(Range::new(3, 1))])]
    #[case("a<`` ` ``>bc", vec![
        Mapped::CharAt(0), Mapped::Text(Range::new(5, 1)),
        Mapped::CharAt(10), Mapped::NextChar])]
    #[case("a<` b", vec![
        Mapped::CharAt(0), Mapped::Text(Range::new(4, 1))])]
    #[case("a\n<`b`>", vec![
        Mapped::CharAt(0), Mapped::LineFeed, Mapped::Text(Range::new(4, 1))])]
    #[case("a\n <`b`>", vec![
        Mapped::CharAt(0), Mapped::LineFeed,
        Mapped::BlankAtLineBeginning(Range::new(2, 1)),
        Mapped::Text(Range::new(5, 1))])]
    #[timeout(time::Duration::from_secs(1))]
    fn it_works(#[case] input: &str, #[case] expected: Vec<Mapped>) {
        let global_parser = global::Parser::new(input.as_bytes(), 0);
        let global_mapper = GlobalEventStreamMapper::new(input.as_bytes(), global_parser);

        let actual: Vec<_> = global_mapper.collect();

        assert_eq!(expected, actual);
    }
}
