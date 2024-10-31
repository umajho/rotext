import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";
import { registerCustomElementForAnkorWidgetDicexp } from "@rotext/solid-components/internal";

import { EvaluatingWorkerManager } from "@dicexp/naive-evaluator-in-worker";

import DicexpEvaluatorWorker from "./dicexp-naive-evaluator.worker?worker";

import { BACKGROUND_COLOR, baseStyleProviders } from "./shared-thingy";

export function registerCustomElementsForDicexp() {
  registerCustomElementForStepsRepresentation("steps-representation");
  registerCustomElementForAnkorWidgetDicexp("ro-widget-dicexp", {
    baseStyleProviders,
    backgroundColor: BACKGROUND_COLOR,
    evaluatorProvider: {
      default: () => {
        const createWorker = () => new DicexpEvaluatorWorker();
        return new Promise(
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
        );
      },
    },
    Loading: () => "loading…",
    ErrorAlert: (props) => <div>{JSON.stringify(props)}</div>,
    tagNameForStepsRepresentation: "steps-representation",
  });
  registerCustomElementForAnkorWidgetDicexp("ro-widget-dicexp-no-runtime", {
    baseStyleProviders,
    backgroundColor: BACKGROUND_COLOR,
    Loading: () => "loading…",
    ErrorAlert: (props) => <div>{JSON.stringify(props)}</div>,
    tagNameForStepsRepresentation: "steps-representation",
  });
}
