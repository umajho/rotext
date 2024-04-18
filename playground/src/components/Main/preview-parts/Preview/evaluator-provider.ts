import {
  createWorkerByImportURLs,
  EvaluatingWorkerManager,
} from "@dicexp/naive-evaluator-in-worker";
import dicexpImportURL from "@dicexp/naive-evaluator/essence/for-worker?url";
import builtinScopeImportURL from "@dicexp/naive-evaluator-builtins/essence/builtin-scope?url";

const createWorker = () =>
  createWorkerByImportURLs(
    (new URL(dicexpImportURL, window.location.href)).href,
    (new URL(builtinScopeImportURL, window.location.href)).href,
  );

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
