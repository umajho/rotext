import {
  Component,
  createEffect,
  createSignal,
  on,
  Show,
  Suspense,
} from "solid-js";
import { RouteSectionProps, useMatch, useNavigate } from "@solidjs/router";
import { Portal } from "solid-js/web";
import { createBreakpoints } from "@solid-primitives/media";

import { HiSolidArrowTopRightOnSquare, HiSolidBars3 } from "solid-icons/hi";

import { SUPPORTS_DVH } from "@rotext/web-utils";

import { Button, Dropdown, DropdownItem, Loading } from "./ui/mod";

import preflight from "../preflight.css?inline";

const navMenuBreakpoints = createBreakpoints({
  "asSidebar": "768px",
});

export const Root: Component<RouteSectionProps> = (props) => {
  const height = SUPPORTS_DVH ? "h-[100dvh]" : "h-screen";

  const [shouldShowNavMenu, setShouldShowNavMenu] = createSignal(true);
  const [shouldShowNavMenuOnWideScreen, setShouldShowNavMenuOnWideScreen] =
    createSignal(true);
  const isDrawerOpen = () =>
    !navMenuBreakpoints.asSidebar && shouldShowNavMenu();
  createEffect(on([() => navMenuBreakpoints.asSidebar], () => {
    if (navMenuBreakpoints.asSidebar) {
      setShouldShowNavMenu(
        shouldShowNavMenu() || shouldShowNavMenuOnWideScreen(),
      );
    } else {
      setShouldShowNavMenuOnWideScreen(shouldShowNavMenu());
      // 在 nav menu 要以抽屉的形式展现时，先将其关上。因为在访问者缩小界面时，
      // 往往不会想让抽屉开着。
      setShouldShowNavMenu(false);
    }
  }));

  return (
    <>
      <Portal>
        {/* XXX: 不能放到 index.tsx 里，否则 vite dev 服务器会无限循环。 */}
        <style id="preflight">{preflight}</style>
      </Portal>
      <Show when={isDrawerOpen()}>
        <div
          class={`absolute top-0 z-10 w-full ${height} bg-black opacity-50 cursor-pointer`}
          onClick={() => setShouldShowNavMenu(false)}
        />
      </Show>
      <div class={`flex flex-col ${height} bg-base-300`}>
        <nav class="sticky top-0 z-30 w-full py-2 sm:p-2">
          <NavBar
            shouldShowNavMenu={shouldShowNavMenu()}
            setShouldShowNavMenu={setShouldShowNavMenu}
          />
        </nav>
        <div class="flex h-full">
          <Show when={shouldShowNavMenu()}>
            <nav
              class={`relative justify-center ${
                isDrawerOpen() ? "w-0" : "w-64"
              }`}
            >
              <div class="absolute z-20 max-h-full h-fit w-60 m-2 overflow-y-scroll bg-base-100 rounded-box">
                <NavMenu
                  onClickMenuItem={() => !navMenuBreakpoints.asSidebar &&
                    setShouldShowNavMenu(false)}
                />
              </div>
            </nav>
          </Show>
          <main class="flex-1">
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
      </div>
    </>
  );
};

const NavBar: Component<{
  shouldShowNavMenu: boolean;
  setShouldShowNavMenu: (value: boolean) => void;
}> = (props) => {
  return (
    <div class="navbar bg-base-100 shadow-xl rounded-box">
      <div class="flex-1 flex items-center">
        <Button
          type={props.shouldShowNavMenu ? "neutral" : "ghost"}
          onClick={() => props.setShouldShowNavMenu(!props.shouldShowNavMenu)}
        >
          <HiSolidBars3 size={24} />
        </Button>
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

const NavMenu: Component<{
  onClickMenuItem: () => boolean | void;
}> = (props) => {
  const navigate = useNavigate();
  const matchPlayground = useMatch(() => "/playground");

  return (
    <ul class="menu w-full">
      <p class="menu-title">导航</p>
      <li>
        <a
          class={`${matchPlayground() ? "active" : ""}`}
          onClick={(ev) => {
            if (isActive(ev)) return;
            props.onClickMenuItem();
            navigate("/playground", { replace: true });
          }}
        >
          实验场
        </a>
      </li>
    </ul>
  );
};

function isActive(ev: { currentTarget: HTMLElement }): boolean {
  return ev.currentTarget.classList.contains("active");
}
