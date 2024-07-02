import { EvaluatingWorkerManager } from "@dicexp/naive-evaluator-in-worker";

import DicexpEvaluatorWorker from "./dicexp-naive-evaluator.worker?worker";

const createWorker = () => new DicexpEvaluatorWorker();

export const evaluatorProvider = {
  default: () =>
    new Promise<EvaluatingWorkerManager>(
      (resolve) => {
        let resolved = false;
        const workerManager = new EvaluatingWorkerManager(
          createWorker,
          (ready) => {
            if (resolved || !ready) return;
            resolve(workerManager);
            resolved = true;
          },
          {
            newEvaluatorOptions: {
              randomSourceMaker: "xorshift7",
            },
          },
        );
      },
    ),
};
