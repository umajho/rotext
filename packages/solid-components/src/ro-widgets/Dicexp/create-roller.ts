import { createSignal } from "solid-js";

import type {
  EvaluatingWorkerManager,
  EvaluationResultForWorker,
} from "@dicexp/evaluating-worker-manager";

type RuntimeLoadingStatus = "short" | "long" | null;

export function createRoller(opts: {
  evaluatorProvider: () => Promise<EvaluatingWorkerManager<any>>;
}) {
  const [rtmLoadingStatus, setRtmLoadingStatus] = //
      createSignal<RuntimeLoadingStatus>(null),
    [isRolling, setIsRolling] = createSignal(false),
    [result, setResult] = createSignal<EvaluationResultForWorker | null>(null);

  async function roll(code: string) {
    if (isRolling()) return;
    setIsRolling(true);
    setResult(null);

    setRtmLoadingStatus("short");
    const cID = //
      setTimeout(() => rtmLoadingStatus() && setRtmLoadingStatus("long"), 100);
    let evaluator: EvaluatingWorkerManager<any> | undefined;
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

    const result = await evaluator.evaluate(code, {
      execute: { topLevelScopeName: "standard" },
    });

    evaluator.destroy();

    setResult(result);
    setIsRolling(false);
  }

  return {
    rtmLoadingStatus,
    isRolling,
    result,
    setResult,
    roll,
  };
}
