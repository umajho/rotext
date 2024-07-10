import { nodeResolve } from "@rollup/plugin-node-resolve";
import typescript from "@rollup/plugin-typescript";
import { dts } from "rollup-plugin-dts";
import del from "rollup-plugin-delete";

/** @type {import('rollup').RollupOptions[]} */
export default [
  {
    input: "./lib.ts",
    output: {
      dir: "./dist",
      format: "esm",
    },
    plugins: [
      nodeResolve(),
      typescript(),
    ],
  },
  {
    input: "./dist/_dts/lib.d.ts",
    output: {
      file: "./dist/lib.d.ts",
    },
    plugins: [
      nodeResolve(),
      dts(),
      del({ hook: "buildEnd", targets: "./dist/_dts" }),
    ],
  },
];
