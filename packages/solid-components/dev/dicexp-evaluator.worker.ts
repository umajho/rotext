import { asScope, startWorkerServer } from "dicexp";
import * as builtins from "@dicexp/builtins";

export const scopes = {
  "standard": asScope([builtins.operatorScope, builtins.functionScope]),
} as const;

startWorkerServer(scopes);
