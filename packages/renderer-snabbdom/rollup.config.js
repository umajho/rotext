import typescript from "@rollup/plugin-typescript";

export default {
  input: "./lib.ts",
  output: {
    dir: "./dist",
    format: "esm",
  },
  plugins: [typescript()],
  external: ["snabbdom", "rotext-renderer-core"],
};
