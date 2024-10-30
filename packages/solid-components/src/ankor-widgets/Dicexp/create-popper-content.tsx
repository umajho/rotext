import { Component, createSignal, Match, Show, Switch } from "solid-js";

import * as Ankor from "ankor";

import { HorizontalRule, PinButton } from "../support/mod";
import { ProcessedProperties } from "./props-for-create-dicexp-component";
import {
  ErrorAlertComponent,
  LoadingComponent,
  StepsRepresentationComponent,
} from "./external-components";

import { errorKindToText } from "./evaluation";

export function createPopperContent(opts: {
  code: () => string;
  processedProperties: ProcessedProperties;

  Loading: LoadingComponent;
  ErrorAlert: ErrorAlertComponent;
  StepsRepresentation: StepsRepresentationComponent;
}): Component<Ankor.PopperContentProperties> {
  return (props) => {
    const { rolling, resultDisplaying } = opts.processedProperties;

    const [showsMoreInExtraInfo, setShowsMoreInExtraInfo] = //
      createSignal(false);

    return (
      <div class="flex flex-col min-w-[12rem] font-sans text-gray-400">
        <div class="flex justify-between items-center px-2 leading-6">
          <div class="flex items-center gap-2">
            <PinButton
              displayMode={props.displayMode}
              onClick={props.handlerForClickOnPinIcon}
              onTouchEnd={props.handlerForTouchEndOnPinIcon}
            />
            <span>掷骰</span>
          </div>
        </div>
        <HorizontalRule color="white" />
        <div style={{ padding: "0.5rem 0.5rem 0 0.5rem" }}>
          <div style={{ padding: "0 0.5rem 0 0.5rem" }}>
            <Switch>
              <Match when={resultDisplaying?.summary()}>
                <Show when={resultDisplaying!.error()}>
                  {(resultError) =>
                    ( // 如果不用 IIFE，resultError() 在变成 null 时仍然会触发
                      // opts.ErrorAlert 的更新，导致后者内部收到 null 作为 error
                      // 的属性值。
                      (e) =>
                        e && (
                          <opts.ErrorAlert
                            kindText={e.kind && errorKindToText(e.kind)}
                            message={e.message}
                          />
                        )
                    )(resultError())}
                </Show>
                <Show when={resultDisplaying!.repr()}>
                  {(resultRepr) => (
                    <>
                      <div class="text-gray-300 my-1">
                        <code class="text-sm font-mono">
                          {opts.code()}
                        </code>
                        {" ➔"}
                      </div>
                      <opts.StepsRepresentation repr={resultRepr()} />
                    </>
                  )}
                </Show>
              </Match>
              <Match when={rolling?.isRolling()}>
                <div class="flex justify-center items-center">
                  <opts.Loading />
                </div>
              </Match>
              <Match when={true}>
                输入变更…
              </Match>
            </Switch>
          </div>
          <Show
            when={resultDisplaying?.statistics() ||
              resultDisplaying?.environment()}
            fallback={<div style={{ height: "0.5rem" }} />}
          >
            <div class="flex flex-col items-end pb-[0.5rem] text-xs text-slate-300">
              <div>
                <Show
                  when={resultDisplaying!.statistics()?.timeConsumption}
                >
                  {(timeConsumption) =>
                    (() => {
                      const location = resultDisplaying!.location();
                      switch (location) {
                        case null:
                          return "";
                        case "local":
                          return "本地";
                        case "server":
                          return "服务器";
                        default:
                          return "？";
                      }
                    })() +
                    `耗时≈${timeConsumption().ms}ms`}
                </Show>
                <Show
                  when={!showsMoreInExtraInfo() &&
                    resultDisplaying!.environment()}
                >
                  {" "}
                  <span
                    class="underline cursor-pointer select-none"
                    onClick={() => setShowsMoreInExtraInfo(true)}
                  >
                    …
                  </span>
                </Show>
              </div>
              <Show
                when={showsMoreInExtraInfo() &&
                  resultDisplaying!.environment()}
              >
                {(environment) => (
                  <>
                    <div>{`求值器=${environment()[0]}`}</div>
                    <div>{`运行时信息=${environment()[1]}`}</div>
                  </>
                )}
              </Show>
            </div>
          </Show>
        </div>
      </div>
    );
  };
}
