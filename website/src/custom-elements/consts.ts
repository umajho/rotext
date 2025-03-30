import { Extension, TagNameMap } from "@rotext/wasm-bindings-adapter";

export const TAG_NAME_MAP = {
  block_call_error: "x-block-call-error",
  scratch_off: "x-scratch-off",
  ref_link: "x-ref-link",
  dicexp: "x-dicexp",
  collapse: "x-collapse",
  code_block: "x-code-block",
  callout: "x-callout",
  wiki_link: "x-wiki-link",
} satisfies TagNameMap & { [k: string]: string };

export const BLOCK_EXTENSION_LIST: Extension[] = [
  {
    ElementMapper: {
      name: "Div",
      tag_name: "div",
      variant: null,
      parameters: {
        "1": { Real: { is_optional: false, mapping_to: "UnnamedSlot" } },
      },
      verbatim_parameters: {},
    },
  },
  {
    ElementMapper: {
      name: "Collapse",
      tag_name: TAG_NAME_MAP.collapse,
      variant: null,
      parameters: {
        "1": { Real: { is_optional: false, mapping_to: "UnnamedSlot" } },
      },
      verbatim_parameters: {
        "title": { Real: { is_optional: true, mapping_to_attribute: "title" } },
        "标题": { Alias: "title" },
        "open": {
          Real: {
            is_optional: true,
            mapping_to_attribute: "open-by-default",
          },
        },
        "展开": { Alias: "open" },
      },
    },
  },
  { Alias: { name: "折叠", to: "Collapse" } },
  ...([
    ["Note", "注"],
    ["Tip", "提示"],
    ["Important", "重要"],
    ["Warning", "警告"],
    ["Caution", "当心"],
  ].flatMap((names): Extension[] => {
    return [
      {
        ElementMapper: {
          name: names[0]!,
          tag_name: TAG_NAME_MAP.callout,
          variant: names[0]!.toLowerCase(),
          parameters: {
            "1": { Real: { is_optional: false, mapping_to: "UnnamedSlot" } },
          },
          verbatim_parameters: {},
        },
      },
      ...names.slice(1).map((name): Extension => ({
        Alias: { name, to: names[0]! },
      })),
    ];
  })),
];

export const PROSE_CLASS = "tuan-prose";

export const CLASSES_FOR_NAVIGATION_ACTION = {
  enabled: "font-bold text-blue-500 hover:text-blue-700",
  disabled: "text-gray-600",
} as const;
