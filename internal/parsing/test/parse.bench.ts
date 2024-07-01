import { bench, describe } from "vitest";

import { parse } from "../lib";

import fs from "node:fs/promises";
import path from "node:path";

const FIXTURE_DOC_INTRODUCTION = await fs.readFile(
  path.join(__dirname, "./fixtures/rotext入门.rotext"),
  { encoding: "utf8" },
);

describe("解析文档 | case 1: 入门文档", () => {
  bench("默认", () => {
    parse(FIXTURE_DOC_INTRODUCTION);
  });
  bench("记录位置信息", () => {
    parse(FIXTURE_DOC_INTRODUCTION, { recordsLocation: true });
  });
});
