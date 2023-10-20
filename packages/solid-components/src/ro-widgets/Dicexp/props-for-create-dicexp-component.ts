import styles from "./DicexpPreview.module.scss";

import { Accessor, createMemo } from "solid-js";

import type { ExecutionAppendix, Repr } from "dicexp";

import { createRoller, RuntimeLoadingStatus } from "./create-roller";
import { summarizeValue } from "./value-summary";
import {
  CreateDicexpComponentOptions,
  DicexpEvaluation,
  Properties,
} from "./create-dicexp-component";

export interface ProcessedProperties {
  rolling?: {
    roll: (code: string) => Promise<void>;
    rtmLoadingStatus: Accessor<RuntimeLoadingStatus>;
    isRolling: Accessor<boolean>;
  };
  resultDisplaying?: {
    summary: () => { text: string; textClass?: string } | null;
    error: () => Error | null;
    repr: () => Repr | null;
    environment: () =>
      | NonNullable<DicexpEvaluation["environment"]>
      | null;
    statistics: () => NonNullable<DicexpEvaluation["statistics"]> | null;
    location: () => NonNullable<DicexpEvaluation["location"]> | null;

    clear?: () => void;
  };
}

export function processProps(
  outerProps: Properties,
  opts: CreateDicexpComponentOptions,
): ProcessedProperties {
  if (opts.evaluatorProvider && !outerProps.evaluation) {
    const roller = createRoller({
      evaluatorProvider: opts.evaluatorProvider.default,
    });

    const appendix = createMemo((): ExecutionAppendix | null => {
      const result = roller.result();
      if (result?.[0] === "ok") {
        return result[2];
      } else if (result?.[0] === "error" && result[1] === "execute") {
        return result[3];
      }
      return null;
    });

    return {
      rolling: {
        roll: roller.roll,
        rtmLoadingStatus: roller.rtmLoadingStatus,
        isRolling: roller.isRolling,
      },
      resultDisplaying: {
        summary: () => {
          const result = roller.result();
          if (!result) return null;

          if (result[0] !== "ok") {
            return {
              text: "错误！",
              textClass: styles["text-color-error"]!,
            };
          }

          const summary = summarizeValue(result[1]);
          if (summary === "too_complex") {
            return {
              text: "暂不支持显示的复杂值。",
              textClass: styles["text-color-warning"]!,
            };
          }
          return { text: summary[1] };
        },
        error: () => {
          const result = roller.result();
          if (result?.[0] === "error" /* && result_[1] !== "execute" */) {
            return result[2];
          }
          return null;
        },
        repr: () => appendix()?.representation ?? null,
        environment: roller.environment,
        statistics: () => appendix()?.statistics ?? null,
        location: () => "local",

        clear: roller.clear,
      },
    };
  } else if (outerProps.evaluation) {
    return {
      resultDisplaying: {
        summary: () => {
          const resultSum = outerProps.evaluation!.result;
          if (resultSum === "error" || resultSum[0] === "error") {
            return {
              text: "错误！",
              textClass: styles["text-color-error"]!,
            };
          } else if (resultSum[0] === "value") {
            const summary = summarizeValue(resultSum[1]);
            if (summary === "too_complex") {
              return ({
                text: "暂不支持显示的复杂值。",
                textClass: styles["text-color-warning"]!,
              });
            } else {
              return ({ text: summary[1] });
            }
          } else {
            resultSum[0] satisfies "value_summary";
            return ({ text: resultSum[1] });
          }
        },
        error: () => {
          const resultSum = outerProps.evaluation!.result;
          if (Array.isArray(resultSum) && resultSum[0] === "error") {
            // TODO: 应该让 ErrorAlert 本身支持 string
            return typeof resultSum[1] === "string"
              ? new Error(resultSum[1])
              : resultSum[1];
          }
          return null;
        },
        repr: () => outerProps.evaluation?.repr ?? null,
        environment: () => outerProps.evaluation?.environment ?? null,
        statistics: () => outerProps.evaluation?.statistics ?? null,
        location: () => outerProps.evaluation?.location ?? null,
      },
    };
  }

  return {};
}
