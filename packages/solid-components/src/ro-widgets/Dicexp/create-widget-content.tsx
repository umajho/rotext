import styles from "./DicexpPreview.module.scss";

import { Component, createSignal, Match, Show, Switch } from "solid-js";

import { WidgetContentProperties } from "../../ro-widget-core/create-widget-component";

import { HorizontalRule, PinButton } from "../support/mod";
import { ProcessedProperties } from "./props-for-create-dicexp-component";
import {
  ErrorAlert,
  Loading,
  StepsRepresentation,
} from "./external-components";

export function createWidgetContent(opts: {
  code: string;
  processedProperties: ProcessedProperties;

  Loading: Loading;
  ErrorAlert: ErrorAlert;
  StepsRepresentation: StepsRepresentation;
}): Component<WidgetContentProperties> {
  return (props) => {
    const { rolling, resultDisplaying } = opts.processedProperties;

    const [showsMoreInExtraInfo, setShowsMoreInExtraInfo] = //
      createSignal(false);

    return (
      <div class={styles["dicexp-widget-content"]}>
        <div class={styles["header"]}>
          <div class={styles["left-area"]}>
            <PinButton
              displayMode={props.displayMode}
              onClick={props.handlerForClickOnPinIcon}
              onTouchEnd={props.handlerForTouchEndOnPinIcon}
            />
            <span>掷骰</span>
          </div>
        </div>
        <HorizontalRule />
        <div style={{ padding: "0.5rem 0.5rem 0 0.5rem" }}>
          <div style={{ padding: "0 0.5rem 0 0.5rem" }}>
            <Switch>
              <Match when={resultDisplaying?.summary()}>
                <Show when={resultDisplaying!.error()}>
                  {(resultError) => (
                    <opts.ErrorAlert
                      error={resultError()}
                      showsStack={false}
                    />
                  )}
                </Show>
                <Show when={resultDisplaying!.repr()}>
                  {(resultRepr) => (
                    <>
                      <div>
                        <code class={styles["code"]}>
                          {opts.code}
                        </code>
                        {" ➔"}
                      </div>
                      <opts.StepsRepresentation repr={resultRepr()} />
                    </>
                  )}
                </Show>
              </Match>
              <Match when={rolling?.isRolling()}>
                <div class={styles["center-aligner"]}>
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
            <div class={styles["extra-info"]}>
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
                    class={styles["more"]}
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
