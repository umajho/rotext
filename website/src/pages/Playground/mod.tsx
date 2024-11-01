import { Component } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { initializeGlobal } from "../../global";

import MainCard from "./MainCard";

export default (() => {
  initializeGlobal({ currentPageName: null, navigator: useNavigate() });

  return (
    <div class="w-full h-full xl:pr-2">
      <MainCard />
    </div>
  );
}) satisfies Component;
