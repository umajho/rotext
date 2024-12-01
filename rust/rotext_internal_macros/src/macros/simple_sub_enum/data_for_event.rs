use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;

/// 为了确保开发体验，使 formatter 可以正常运作[^1]，这里预先定义了作为变体的各
/// 事件对应的分组。这些定义与枚举 [rotext::events::Event] 中的定义同步，且由宏
/// [simple_sub_enum] 检查是否同步。`ensurer` 系列宏将直接使用这里的数据。
///
/// [^1]: 自动生成的 `ensurer` 系列宏只能使用函数风格，导致 formatter 忽略宏的
/// 内部。
static EVENT_TO_GROUP_RAW: &[(&str, &[&str])] = &[
    ("__Unparsed", &["Block", "InlineInput"]),
    ("Raw", &["Inline", "Blend"]),
    (
        "VerbatimEscaping",
        &["Block", "InlineInput", "Inline", "Blend"],
    ),
    ("NewLine", &["Block", "InlineInput", "Inline", "Blend"]),
    ("Text", &["Block", "Inline", "Blend"]),
    ("ThematicBreak", &["Block", "Blend"]),
    ("EnterParagraph", &["Block", "Blend"]),
    ("EnterHeading1", &["Block", "Blend"]),
    ("EnterHeading2", &["Block", "Blend"]),
    ("EnterHeading3", &["Block", "Blend"]),
    ("EnterHeading4", &["Block", "Blend"]),
    ("EnterHeading5", &["Block", "Blend"]),
    ("EnterHeading6", &["Block", "Blend"]),
    ("EnterBlockQuote", &["Block", "Blend"]),
    ("EnterOrderedList", &["Block", "Blend"]),
    ("EnterUnorderedList", &["Block", "Blend"]),
    ("EnterListItem", &["Block", "Blend"]),
    ("EnterDescriptionList", &["Block", "Blend"]),
    ("EnterDescriptionTerm", &["Block", "Blend"]),
    ("EnterDescriptionDetails", &["Block", "Blend"]),
    ("EnterCodeBlock", &["Block", "Blend"]),
    ("EnterTable", &["Block", "Blend"]),
    ("IndicateCodeBlockCode", &["Block", "Blend"]),
    ("IndicateTableCaption", &["Block", "Blend"]),
    ("IndicateTableRow", &["Block", "Blend"]),
    ("IndicateTableHeaderCell", &["Block", "Blend"]),
    ("IndicateTableDataCell", &["Block", "Blend"]),
    ("ExitBlock", &["Block", "Blend"]),
    ("RefLink", &["Inline", "Blend"]),
    ("Dicexp", &["Inline", "Blend"]),
    ("EnterCodeSpan", &["Inline", "Blend"]),
    ("EnterStrong", &["Inline", "Blend"]),
    ("EnterStrikethrough", &["Inline", "Blend"]),
    ("EnterInternalLink", &["Inline", "Blend"]),
    ("ExitInline", &["Inline", "Blend"]),
];

lazy_static! {
    pub static ref ALL_EVENTS: HashSet<String> = EVENT_TO_GROUP_RAW
        .iter()
        .map(|(variant, _)| variant.to_string())
        .collect();
    pub static ref AVAILABLE_GROUPS: HashSet<String> = EVENT_TO_GROUP_RAW
        .iter()
        .flat_map(|(_, groups)| groups.iter().map(|group| group.to_string()))
        .collect();
    pub static ref GROUP_TO_EVENT: HashMap<String, HashSet<String>> = {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        for (variant, groups) in EVENT_TO_GROUP_RAW {
            for group in groups.iter() {
                map.entry(group.to_string())
                    .or_default()
                    .insert(variant.to_string());
            }
        }
        map
    };
}
