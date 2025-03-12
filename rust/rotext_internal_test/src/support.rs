use core::panic;
use std::{any::Any, panic::RefUnwindSafe};

pub struct GroupedCases<TCase: Case> {
    pub group: &'static str,
    pub cases: Vec<TCase>,
}
impl<TCase: Case + RefUnwindSafe> GroupedCases<TCase> {
    pub fn collect_failed(&self) -> Vec<FailedCase> {
        self.cases
            .iter()
            .enumerate()
            .filter_map(|(i, case)| -> Option<FailedCase> {
                let panic = std::panic::catch_unwind(|| case.assert_ok()).err()?;
                Some(FailedCase {
                    group: self.group,
                    nth_case_in_group: i + 1,
                    nth_case_variant_in_case: None,
                    auto_variant: None,
                    input: case.input().to_string(),
                    reason: FailureReason::Panicked(panic),
                })
            })
            .collect()
    }
}

pub trait Case {
    fn input(&self) -> String;
    fn assert_ok(&self);
}

pub struct FailedCase {
    pub group: &'static str,
    pub nth_case_in_group: usize,
    pub nth_case_variant_in_case: Option<usize>,
    pub auto_variant: Option<&'static str>,
    pub input: String,
    pub reason: FailureReason,
}

pub enum FailureReason {
    Panicked(Box<dyn Any + Send>),
    ToDo,
    Skipped,
}

pub fn report_panicked_cases(cases: Vec<FailedCase>) {
    for case in cases {
        let FailureReason::Panicked(panic) = case.reason else {
            continue;
        };

        print!("=> group={} case={}", case.group, case.nth_case_in_group);
        if let Some(nth) = case.nth_case_variant_in_case {
            print!(" case_variant={}", nth)
        }
        if let Some(variant) = case.auto_variant {
            print!(" auto_variant={}", variant)
        }
        println!();
        println!("-> input:\n{}", case.input);
        let panic_message: String = {
            match panic.downcast::<String>() {
                Ok(str) => *str,
                Err(panic) => match panic.downcast::<&str>() {
                    Ok(str) => str.to_string(),
                    Err(_) => unimplemented!(),
                },
            }
        };
        println!("-> panic:\n{}", panic_message);
        print!("\n\n");
    }
}

/// 生成与空白有关的字符串变体。
///
/// - 将所有的 `␣` 替换为 ` `。（用于 workaround `indoc` 吞掉一行首尾空白的行为。）
/// - 对每个位置的 `␠`，生成替换为 ` ` 与 `\t` 的变体。（用于同时顾及两种空白的情况。）
///
/// NOTE: 生成的变体的数量会随着输入中的 `␠` 的数量呈指数增长。因此限制输入中最多存在 12 个
/// `␠`。
pub fn make_whitespace_variants(input: &str) -> Vec<String> {
    if input.contains(" ") {
        panic!(
            "应该使用 `␣`（空格）或 `␠`（空格或制表符）代替 ` `：“{}”",
            input
        )
    }

    let input = input.replace('␣', " ");

    fn replace_whitespace_characters(input: &str, remain: usize) -> Vec<String> {
        let space_variant = input.replacen('␠', " ", 1);
        let tab_variant = input.replacen('␠', "\t", 1);
        if remain == 0 {
            vec![space_variant, tab_variant]
        } else {
            let mut result = Vec::new();
            result.extend(replace_whitespace_characters(&space_variant, remain - 1));
            result.extend(replace_whitespace_characters(&tab_variant, remain - 1));
            result
        }
    }

    let remain = input.chars().filter(|c| *c == '␠').count();
    if remain > 12 {
        panic!("输入中的`␠`太多了：“{}”", input);
    }
    replace_whitespace_characters(&input, remain)
}
