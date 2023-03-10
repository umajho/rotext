import typescript from "@rollup/plugin-typescript";
import { nodeResolve } from "@rollup/plugin-node-resolve";

export default {
  input: "./lib.ts",
  output: {
    dir: "./dist",
    format: "esm",
  },
  plugins: [typescript(), nodeResolve()],
};
