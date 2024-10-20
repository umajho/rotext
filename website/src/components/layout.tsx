import {
  Component,
  createContext,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Match,
  on,
  Show,
  Suspense,
  Switch,
  useContext,
} from "solid-js";
import {
  RouteSectionProps,
  useLocation,
  useMatch,
  useNavigate,
} from "@solidjs/router";
import { createBreakpoints } from "@solid-primitives/media";

import { HiSolidArrowTopRightOnSquare, HiSolidBars3 } from "solid-icons/hi";

import { Button, Dropdown, DropdownItem, Loading } from "./ui/mod";

import { Navigation } from "../types/navigation";
import { syntaxReferenceIndex } from "../data-sources/syntax-reference";
import { getSyntaxReferencePathOfHeading } from "../utils/syntax-reference";
import { SUPPORTS_DVH } from "../utils/mod";
import { useRotextProcessorsStore } from "../contexts/rotext-processors-store";
import { RotextProcessorName } from "../hooks/rotext-processors-store";

import "../styles/preflight";
import "../styles/tailwind";

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
        <div class="flex-1 flex overflow-y-hidden">
          <nav
            class={[
              "relative justify-center",
              isDrawerOpen() ? "w-0" : "w-[19rem]",
              ...(shouldShowNavMenu() ? [] : ["hidden"]),
            ].join(" ")}
          >
            <div class="absolute z-20 max-h-full h-fit w-72 mx-2 overflow-y-scroll bg-base-100 rounded-box">
              <NavMenu
                onClickMenuItem={() =>
                  !navMenuBreakpoints.asSidebar &&
                  setShouldShowNavMenu(false)}
              />
            </div>
          </nav>
          <main class="flex-1 overflow-x-hidden">
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

const SyntaxReferenceIndexContext = //
  createContext<{ [heading: string]: string }>();

const NavMenu: Component<{
  onClickMenuItem: () => boolean | void;
}> = (props) => {
  const rotextProcessors = useRotextProcessorsStore();

  let selectEl!: HTMLSelectElement;

  const navigate = useNavigate();
  const matchPlayground = useMatch(() => "/playground");

  const [syntaxReferenceNavigation] = createResource(async () => {
    const resp = await fetch(
      "static/generated/syntax-reference/navigation.json",
    );
    return resp.json() as Promise<Navigation>;
  });
  const isSyntaxReferenceReady = () =>
    (!!syntaxReferenceNavigation()) && (!!syntaxReferenceIndex());

  return (
    <ul class="menu w-full">
      <p class="menu-title">解析器 & 渲染器</p>
      <li>
        <select
          ref={selectEl}
          disabled={rotextProcessors?.isBusy ?? true}
          onChange={(ev) =>
            rotextProcessors?.switchProcessor(
              ev.currentTarget.value as RotextProcessorName,
            )}
        >
          <option
            value="rust"
            selected={rotextProcessors?.currentName === "rust"}
          >
            新式（WIP）
          </option>
        </select>
      </li>
      <p class="menu-title">导航</p>
      <li>
        <a
          class={`${matchPlayground() ? "active" : ""}`}
          onClick={(ev) => {
            if (isActive(ev)) return;
            props.onClickMenuItem();
            navigate("/playground");
          }}
        >
          实验场
        </a>
      </li>
      <li>
        <details open>
          <summary>语法参考（WIP）</summary>
          <Show
            when={isSyntaxReferenceReady()}
            fallback={
              <ul>
                <li class="disabled">
                  <div class="flex justify-center disabled">
                    <Loading />
                  </div>
                </li>
              </ul>
            }
          >
            <SyntaxReferenceIndexContext.Provider
              value={syntaxReferenceIndex()}
            >
              <NavMenuList
                navigationList={syntaxReferenceNavigation()!.children ?? []}
              />
            </SyntaxReferenceIndexContext.Provider>
          </Show>
        </details>
      </li>
    </ul>
  );
};

function isActive(ev: { currentTarget: HTMLElement }): boolean {
  return ev.currentTarget.classList.contains("active");
}

const NavMenuList: Component<{
  navigationList: Navigation[];
}> = (props) => {
  return (
    <ul>
      <For each={props.navigationList}>
        {(item) => <NavMenuListItem navigation={item} />}
      </For>
    </ul>
  );
};

const NavMenuListItem: Component<{
  navigation: Navigation;
}> = (props) => {
  return (
    <li>
      <Switch>
        <Match when={props.navigation.children}>
          {(children) => (
            <NavMenuListItemInnerBranch
              name={props.navigation.name}
              children={children()}
            />
          )}
        </Match>
        <Match when={true}>
          <NavMenuListItemInnerLeaf
            navigation={props.navigation}
          />
        </Match>
      </Switch>
    </li>
  );
};

const NavMenuListItemInnerBranch: Component<{
  name: string;
  children: Navigation[];
}> = (props) => {
  return (
    <details open>
      <summary>{props.name}</summary>
      <NavMenuList
        navigationList={props.children}
      />
    </details>
  );
};

const NavMenuListItemInnerLeaf: Component<{
  navigation: Navigation;
}> = (props) => {
  const navigate = useNavigate();
  const location = useLocation();

  const index = useContext(SyntaxReferenceIndexContext)!;

  const heading = () => props.navigation.realName ?? props.navigation.name;
  const path = createMemo(() =>
    getSyntaxReferencePathOfHeading(heading(), { index })
  );

  const match = () =>
    `${decodeURIComponent(location.pathname)}` === path().path;

  return (
    <a
      class={`${match() ? "active" : ""}`}
      onClick={() => navigate(path().pathWithAnchor)}
    >
      {props.navigation.name}
    </a>
  );
};
