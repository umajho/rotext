import { Component } from "solid-js";

import MainCard from "./MainCard";

const Main: Component = () => {
  return (
    <main>
      <div
        class={`
        flex justify-center flex-col items-center
      `}
      >
        <MainCard />
      </div>
    </main>
  );
};
export default Main;
