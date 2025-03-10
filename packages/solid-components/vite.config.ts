import path from "node:path";

import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import dts from "vite-plugin-dts";

const ENTRIES = {
    "src/index.ts": "index.js",
    "src/ankor-widgets/Navigation/mod.ts": "ankor-widgets/Navigation/mod.js",
    "src/ankor-widgets/Dicexp/mod.ts": "ankor-widgets/Dicexp/mod.js",
};

export default defineConfig({
    plugins: [
        solidPlugin(),
        dts({
            include: "src/**/*",
            // 如果为真，会导致除 `index.js` 之外的输出文件没有对应的 `.d.ts` 文件。
            // rollupTypes: true,
        }),
    ],
    build: {
        outDir: "./dist",
        lib: {
            formats: ["es"],
            entry: Object.keys(ENTRIES),
        },
        rollupOptions: {
            output: {
                entryFileNames(chunkInfo) {
                    const entryPath = path.relative(
                        __dirname,
                        chunkInfo.facadeModuleId!,
                    );
                    const entryOutFileName =
                        (ENTRIES as any)[entryPath] as string;
                    return entryOutFileName;
                },
            },
            //external: getDependencies(),
            external(source) {
                return !/^[/.]/.test(source);
            },
        },
    },
});
