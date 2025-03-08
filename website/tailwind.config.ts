import type { Config } from "tailwindcss";

import daisyui from "daisyui";
import typography from "@tailwindcss/typography";

const config: Config = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}",
    "./node_modules/@rotext/solid-components/src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    fontFamily: {
      sans: ["Arial", "Noto Sans Mono CJK SC"],
      serif: ["Times New Roman", "LXGW WenKai"],
      mono: ["Noto Sans Mono CJK SC"],
    },
    extend: {},
  },
  plugins: [typography, daisyui],
  daisyui: {
    themes: ["dark"],
  },
};

export default config;
