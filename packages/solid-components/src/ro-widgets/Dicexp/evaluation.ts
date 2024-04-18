import { JSValue, Repr } from "@dicexp/interface";

export interface DicexpEvaluation {
  /**
   * 求值环境。在重建求值结果时需要用到。
   */
  environment?: [
    /**
     * 求值器的名称。版本是名称的一部分。
     *
     * 如：`"$@0.4.1"` 或者等价的 `"dicexp@0.4.1"`
     */
    evaluatorName: string,
    /**
     * 求值器运行时的信息。求值器应该保证在相同的信息下，求值的结果（包括步骤）总是相同。
     *
     * 比如，对于 dicexp@0.4.1 而言，要满足上述条件，信息要包括：随机数生成方案名、种子数、顶部作用域的路径。
     *
     * 如：`"{r:42,s:"0.4.0"}"`，或者等价的 `"{r:["xorshift7",42],s:["@dicexp/builtins@0.4.0","./essence/standard-soceps","standard"]}"`。
     * （其中，“r” 代表 “Rng (Random number generator)”，“s” 代表 “top level Scope”。）
     */
    runtimeInfo: string,
  ];
  result:
    | ["value", JSValue]
    | ["value_summary", string]
    | "error"
    | ["error", ErrorKind, string];
  repr?: Repr;
  statistics?: {
    timeConsumption?: { ms: number };
  };
  /**
   * 标记求值是在哪里进行的。
   *
   * TODO: 未来实现重建步骤的功能时，以数组类型的计算值存储统计，每项统计新增一个 location
   *       属性，以实现区分原先和重建后的统计内容。
   *      （如果原先本来就带步骤，那数组就只会有一项。）
   */
  location?: "local" | "server";
}

export type ErrorKind = "parse" | "runtime" | "worker_client" | "other";

export function errorKindToText(k: ErrorKind): string {
  switch (k) {
    case "parse":
      return "解析";
    case "runtime":
      return "运行时";
    case "worker_client":
      return "worker 客户端";
    case "other":
      return "其他";
    default:
      return `未知类型（“${k}”）`;
  }
}
