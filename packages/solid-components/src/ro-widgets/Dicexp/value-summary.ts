import type { JSValue } from "dicexp";

const MAX_LIST_SUMMARY_TEXT_LENGTH = 16;

export type ValueSummary =
  | ["exact", string]
  | ["summary", string]
  | "too_complex";

export function summarizeValue(
  value: JSValue,
): ["exact" | "summary", string] | "too_complex" {
  if (typeof value === "number") {
    return ["exact", `${value}`];
  } else if (typeof value === "boolean") {
    return ["exact", value ? "真" : "假"];
  } else if (!Array.isArray(value)) {
    return "too_complex";
  } else if (
    value.some((item) =>
      (typeof item !== "number") && (typeof item !== "boolean")
    )
  ) {
    return "too_complex";
  }

  let listText = "[";
  for (const [i, item] of value.entries()) {
    const itemText = (typeof item === "boolean")
      ? (item ? "真" : "假")
      : `${item}`;
    const lengthAfter = listText.length + itemText.length +
      1 + // "]"
      ((i < value.length - 1) ? 2 : 0); // ", "
    if (lengthAfter > MAX_LIST_SUMMARY_TEXT_LENGTH) {
      listText += "…";
      break;
    } else {
      listText += itemText;
    }
    if (i < value.length - 1) {
      listText += ", ";
    }
  }
  listText += "]";

  let listSummaryText: string | null = null;
  if (value.every((item) => typeof item === "number")) {
    let sum = 0;
    for (const item of value as number[]) {
      sum += item;
      if (sum > Number.MAX_SAFE_INTEGER) {
        listSummaryText = "总和过大";
        break;
      }
    }
    listSummaryText = `合计${sum}`;
  } else if (value.every((item) => typeof item === "boolean")) {
    const trues = value.filter((item) => item).length;
    listSummaryText =
      `${parseFloat((trues / value.length * 100).toFixed(2))}%` +
      // ` (${trues}/${value.length})` +
      " 为真";
  } else {
    listSummaryText = "不同类型混合";
  }

  return ["summary", `${listText} (${listSummaryText})`];
}
