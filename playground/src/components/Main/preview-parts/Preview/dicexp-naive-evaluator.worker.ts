import { startWorkerServer } from "@dicexp/naive-evaluator-in-worker";

import evaluatorMakerImportURL from "@dicexp/naive-evaluator/essence/for-worker?url";
import topLevelScopeImportURL from "@dicexp/naive-evaluator-builtins/essence/builtin-scope?url";

startWorkerServer(evaluatorMakerImportURL, topLevelScopeImportURL);
