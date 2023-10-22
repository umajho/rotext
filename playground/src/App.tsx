import type { Component } from "solid-js";
import { Portal } from "solid-js/web";

import { NavBar } from "./components/layout";
import Main from "./components/Main/mod";
import { SUPPORTS_DVH } from "@rotext/web-utils";

import preflight from "./preflight.css?inline";

const App: Component = () => {
  const minH = SUPPORTS_DVH ? "min-h-[100dvh]" : "min-h-screen";

  return (
    <>
      <Portal>
        {/* XXX: 不能放到 index.tsx 里，否则 vite dev 服务器会无限循环。 */}
        <style id="preflight">{preflight}</style>
      </Portal>
      <div class={`app-container ${minH} bg-base-300`}>
        <nav class="sticky top-0 z-10 w-full py-2 sm:p-2">
          <NavBar />
        </nav>
        <div class="h-4 md:h-8" />
        <Main />
      </div>
    </>
  );
};

export default App;
