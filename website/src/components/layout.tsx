import { Component, Suspense } from "solid-js";
import { RouteSectionProps } from "@solidjs/router";
import { Portal } from "solid-js/web";

import { HiSolidArrowTopRightOnSquare } from "solid-icons/hi";

import { SUPPORTS_DVH } from "@rotext/web-utils";

import { Dropdown, DropdownItem, Loading } from "./ui/mod";

import preflight from "../preflight.css?inline";

export const Root: Component<RouteSectionProps> = (props) => {
  const height = SUPPORTS_DVH ? "h-[100dvh]" : "h-screen";

  return (
    <>
      <Portal>
        {/* XXX: 不能放到 index.tsx 里，否则 vite dev 服务器会无限循环。 */}
        <style id="preflight">{preflight}</style>
      </Portal>
      <div class={`flex flex-col ${height} bg-base-300`}>
        <nav class="sticky top-0 z-10 w-full py-2 sm:p-2">
          <NavBar />
        </nav>
        <main class="h-full">
          <Suspense
            fallback={
              <div class="flex h-full justify-center items-center">
                <Loading />
              </div>
            }
          >
            {props.children}
          </Suspense>
        </main>
      </div>
    </>
  );
};

const NavBar: Component = () => {
  return (
    <div class="navbar bg-base-100 shadow-xl rounded-box">
      <div class="flex-1 flex items-center">
        <Dropdown
          summary={
            <>
              团岛计划
            </>
          }
          labelClass="btn-ghost"
          contentClass="border-[0.5px] border-black"
        >
          <DropdownItem>
            <a>轻量级标记语言 Rotext（当前）</a>
          </DropdownItem>
          <DropdownItem>
            <a href="https://umajho.github.io/dicexp">骰子表达式 Dicexp</a>
          </DropdownItem>
        </Dropdown>
        <a class="btn btn-ghost normal-case text-xl max-sm:p-0">
          <span>
            Rotext
          </span>
        </a>
      </div>
      <div>
        <ul class="menu menu-horizontal px-1">
          <li>
            <a
              class="inline-flex items-center"
              href="https://github.com/umajho/rotext"
            >
              代码
              <HiSolidArrowTopRightOnSquare size={16} />
            </a>
          </li>
        </ul>
      </div>
    </div>
  );
};
