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
