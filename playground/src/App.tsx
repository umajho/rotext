import type { Component } from "solid-js";

import { NavBar } from "./components/layout";
import Main from "./components/Main/mod";
import { SUPPORTS_DVH } from "@rotext/web-utils";

const App: Component = () => {
  const minH = SUPPORTS_DVH ? "min-h-[100dvh]" : "min-h-screen";

  return (
    <div class={`app-container ${minH} bg-base-300`}>
      <nav class="sticky top-0 z-10 w-full py-2 sm:p-2">
        <NavBar />
      </nav>
      <div class="h-4 md:h-8" />
      <Main />
    </div>
  );
};

export default App;
