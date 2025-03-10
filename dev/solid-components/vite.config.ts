import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  resolve: {
    alias: {
      src: "/src",
    },
  },
  plugins: [
    tailwindcss(),
    solidPlugin(),
    {
      name: "Reaplace env variables",
      transform(code, id) {
        if (id.includes("node_modules")) {
          return code;
        }
        return code
          .replace(/process\.env\.SSR/g, "false")
          .replace(/process\.env\.DEV/g, "true")
          .replace(/process\.env\.PROD/g, "false")
          .replace(/process\.env\.NODE_ENV/g, '"development"')
          .replace(/import\.meta\.env\.SSR/g, "false")
          .replace(/import\.meta\.env\.DEV/g, "true")
          .replace(/import\.meta\.env\.PROD/g, "false")
          .replace(/import\.meta\.env\.NODE_ENV/g, '"development"');
      },
    },
  ],
  server: {
    port: 3000,
  },
  build: {
    target: "esnext",
  },
  optimizeDeps: {
    // exclude: ["@rolludejo/internal-web-shared"],
    // 实在不知道是哪里导致投玩骰子后，悬浮框和箭头右边结果都出来了，但 “正在试投” 的文字就不
    // 会消失，骰子图标也一直转，浏览器报错找不到 `React`。直接完全禁用依赖优化得了。
    disabled: true,
  },
});
