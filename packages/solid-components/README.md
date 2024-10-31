# @rotext/solid-components

最初 fork 自 [dicexp] [此次 commit] 下的
`packages/solid-components`，基于此清除了原项目相关的代码，并调整了部分配置。往前可以追溯至
[solid-lib-starter]。（详见 dicexp 对应位置的记录。）

本包将 `solid-js`、`solid-icons` 和 `solid-element` 作为 peer 依赖。

[dicexp]: https://github.com/umajho/dicexp
[此次 commit]: https://github.com/umajho/dicexp/commit/3a4eb123ff3fb57897f13118ae103eeb4666e1ba
[solid-lib-starter]: https://github.com/solidjs-community/solid-lib-starter

## 额外准备工作

> [!IMPORTANT]
>
> 本章节所述内容为让本包正常工作的必须步骤。

### 1. 让 tailwind 扫描本包的代码

在使用本包的项目的 tailwind 配置文件（假设为 `tailwind.config.ts`）中添加
`content` 项：

```diff
  content: [
    …,
+   "./node_modules/@rotext/solid-components/src/**/*.{js,ts,jsx,tsx}",
  ],
```

### 2. 在调用 `registerCustomElementFor*` 时，提供包含 tailwind 生成样式的 `StyleProvider`

通过第二个参数（`opts`）的 `baseStyleProviders` 字段提供。

`StyleProvider` 来自 [`@rolludejo/internal-web-shared`]。

[`@rolludejo/internal-web-shared`]: https://www.npmjs.com/package/@rolludejo/internal-web-shared
