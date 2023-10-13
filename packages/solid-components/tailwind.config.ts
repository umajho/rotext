import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./dev/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}",
    "./src/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}",
  ],
  darkMode: "class",
  theme: {
    extend: {},
  },
};

export default config;
