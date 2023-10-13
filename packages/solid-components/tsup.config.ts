import { defineConfig } from "tsup";
import * as preset from "tsup-preset-solid";

import { postcssModules, sassPlugin } from "esbuild-sass-plugin";
import postcss from "postcss";
import autoprefixer from "autoprefixer";
import tailwindcss from "tailwindcss";

const preset_options: preset.PresetOptions = {
  // array or single object
  entries: [
    // default entry (index)
    ...[
      { entry: "src/index.tsx" },
      { entry: "src/ro-widget-core/mod.tsx", name: "widget-core" },
      { entry: "src/ro-widgets/RefLink/mod.tsx", name: "RefLink" },
      { entry: "src/ro-widgets/Dicexp/mod.tsx", name: "Dicexp" },
    ].map(({ entry, name }) => ({
      // entries with '.tsx' extension will have `solid` export condition generated
      entry,
      name,
      // will generate a separate development entry
      dev_entry: true,
    })),
  ],
  // Set to `true` to remove all `console.*` calls and `debugger` statements in prod builds
  drop_console: true,
  // Set to `true` to generate a CommonJS build alongside ESM
  // cjs: true,
};

const CI = process.env["CI"] === "true" ||
  process.env["GITHUB_ACTIONS"] === "true" ||
  process.env["CI"] === '"1"' ||
  process.env["GITHUB_ACTIONS"] === '"1"';

export default defineConfig((config) => {
  const watching = !!config.watch;

  const parsed_options = preset.parsePresetOptions(preset_options, watching);

  if (!watching && !CI) {
    const package_fields = preset.generatePackageExports(parsed_options);

    const exportKeys = Object.keys(package_fields.exports);
    if (exportKeys.length && !exportKeys[0]!.startsWith(".")) {
      package_fields.exports = {
        ".": package_fields.exports,
      };
    }

    // 让 workspace 内的其他项目能直接引入
    package_fields.exports["./internal"] = "./internal.ts";

    console.log(
      `package.json: \n\n${JSON.stringify(package_fields, null, 2)}\n\n`,
    );

    // will update ./package.json with the correct export fields
    preset.writePackageJson(package_fields);
  }

  const tsUpOptions = preset.generateTsupOptions(parsed_options)
    .map((opt) => {
      // opt.loader ??= {};
      // // 如果只用 CSS 不用 SCSS 的话：
      // // https://github.com/egoist/tsup/issues/536#issuecomment-1752121594
      // opt.loader[".css"] = "local-css";

      const postcssPlugins = [tailwindcss as any, autoprefixer];

      opt.esbuildPlugins ??= [];
      opt.esbuildPlugins.push(...[
        sassPlugin({
          filter: /module\.[^.]+$/,
          type: "style",
          transform: postcssModules({}, postcssPlugins),
        }),
        sassPlugin({
          type: "css-text",
          transform: async (source) => {
            return (await postcss(postcssPlugins).process(source)).css;
          },
        }),
      ]);

      return opt;
    });
  return tsUpOptions;
});
