import { createSignal } from "solid-js";

import type {
  EvaluatingWorkerManager,
} from "@dicexp/naive-evaluator-in-worker";
import type { EvaluationResult } from "@dicexp/interface";

import { DicexpEvaluation } from "./evaluation";

export type RuntimeLoadingStatus = "short" | "long" | null;

export function createRoller(opts: {
  evaluatorProvider: () => Promise<EvaluatingWorkerManager>;
}) {
  const [rtmLoadingStatus, setRtmLoadingStatus] = //
      createSignal<RuntimeLoadingStatus>(null),
    [isRolling, setIsRolling] = createSignal(false),
    [result, setResult] = createSignal<EvaluationResult | null>(null),
    [environment, setEnvironment] = //
      createSignal<NonNullable<DicexpEvaluation["environment"]> | null>(
        null,
      );

  async function roll(code: string) {
    if (isRolling()) return;
    setIsRolling(true);
    setResult(null);
    setEnvironment(null);

    setRtmLoadingStatus("short");
    const cID = //
      setTimeout(() => rtmLoadingStatus() && setRtmLoadingStatus("long"), 100);
    let evaluator: EvaluatingWorkerManager | undefined;
    try {
      evaluator = await opts.evaluatorProvider();
    } catch (e) {
      const reason = (e instanceof Error) ? e.message : `e`;
      setResult(["error", "other", new Error(`加载运行时失败：${reason}`)]);
    }
    setRtmLoadingStatus(null);
    clearTimeout(cID);

    if (!evaluator) {
      setResult(null);
      setIsRolling(false);
      return;
    }

    const seed = crypto.getRandomValues(new Uint32Array(1))[0]!;
    const result = await evaluator.evaluateRemote(code, {
      execution: { seed },
    });

    evaluator.destroy();

    setResult(result);
    setEnvironment(["?", JSON.stringify({ r: seed, s: "?" })]);
    setIsRolling(false);
  }

  return {
    rtmLoadingStatus,
    isRolling,
    result,
    environment,
    roll,
    clear: () => {
      setResult(null);
      setEnvironment(null);
    },
  };
}
