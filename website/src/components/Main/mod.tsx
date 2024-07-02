import { Component } from "solid-js";

import MainCard from "./MainCard";

const Main: Component = () => {
  return (
    <div
      class={`
        flex justify-center flex-col items-center
      `}
    >
      <MainCard />
    </div>
  );
};
export default Main;
