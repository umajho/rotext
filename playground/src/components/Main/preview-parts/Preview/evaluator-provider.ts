import {
  createWorkerByImportURLs,
  EvaluatingWorkerManager,
} from "@dicexp/evaluating-worker-manager";
import dicexpImportURL from "dicexp/essence/for-worker?url";
import scopesImportURL from "@dicexp/builtins/essence/standard-scopes?url";

export function evaluatorProvider() {
  const createWorker = () =>
    createWorkerByImportURLs(
      (new URL(dicexpImportURL, window.location.href)).href,
      (new URL(scopesImportURL, window.location.href)).href,
    );
  return new Promise<EvaluatingWorkerManager<any>>(
    (resolve) => {
      let resolved = false;
      const workerManager = new EvaluatingWorkerManager(
        createWorker,
        (ready) => {
          if (resolved || !ready) return;
          resolve(workerManager);
          resolved = true;
        },
      );
    },
  );
}
