import { createSignal } from "solid-js";

import type {
  EvaluatingWorkerManager,
  EvaluationResultForWorker,
} from "dicexp";

type RuntimeLoadingStatus = "short" | "long" | null;

export function createRoller(opts: {
  dicexpImporter: () => Promise<typeof import("dicexp")>;
  EvaluatingWorker: new () => Worker;
}) {
  const { dicexpImporter, EvaluatingWorker } = opts;

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
    let dicexp: typeof import("dicexp") | undefined;
    try {
      dicexp = await dicexpImporter();
    } catch (e) {
      const reason = (e instanceof Error) ? e.message : `e`;
      setResult(["error", "other", new Error(`加载运行时失败：${reason}`)]);
    }
    setRtmLoadingStatus(null);
    clearTimeout(cID);

    if (!dicexp) {
      setResult(null);
      setIsRolling(false);
      return;
    }

    const workerManager = await new Promise<EvaluatingWorkerManager<any>>(
      (resolve) => {
        let resolved = false;
        const workerManager = new dicexp!.EvaluatingWorkerManager(
          () => new EvaluatingWorker(),
          (ready) => {
            if (resolved || !ready) return;
            resolve(workerManager);
            resolved = true;
          },
        );
      },
    );
    const result = await workerManager.evaluate(code, {
      execute: { topLevelScopeName: "standard" },
    });

    workerManager.destroy();

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
