import { bench, describe } from "vitest";

import { parse } from "../lib";

import * as examples from "@rotext/example-documentations";

describe("解析文档 | case 1: 入门文档", () => {
  bench("默认", () => {
    parse(examples.introduction);
  });
  bench("记录位置信息", () => {
    parse(examples.introduction, { recordsLocation: true });
  });
});
