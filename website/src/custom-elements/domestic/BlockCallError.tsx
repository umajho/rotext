import { customElement, getCurrentElement } from "solid-element";
import { Component, createMemo, Index, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createBlockCallErrorComponent(): Component<{
  "call-type": "transclusion" | "extension" | "";
  "call-name": string;
  "error-type": string;
  "error-value": string | null;
}> {
  return (props) => {
    const currentElement = getCurrentElement();

    const what = createMemo(() => {
      switch (props["call-type"]) {
        case "transclusion":
          return "嵌入包含";
        case "extension":
          return "扩展";
        default:
          return "未知";
      }
    });

    const message = () => {
      switch (props["error-type"]) {
        case "TODO":
          return <p>TODO: 实现渲染</p>;
        case "UnknownCallee":
          return <p>{`没有找到这个名称的${what()}。`}</p>;
        case "BadParameters":
          return <BadParametersErrorMessage valueRaw={props["error-value"]!} />;
        default:
          let errorType = props["error-type"];
          let errorValue = props["error-value"];
          if (typeof errorValue === "string") {
            return <p>{`${errorType} (${errorValue})`}</p>;
          } else {
            return <p>{errorType}</p>;
          }
      }
    };

    onMount(() => {
      for (const p of [styleProviderForPreflight, styleProviderForTailwind]) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <div class="p-4 my-1 border border-red-500 border-dashed text-red-500">
        <p class="font-bold pb-4">
          {`调用${what()}「`}
          <Name name={props["call-name"]} />
          {`」失败：`}
        </p>
        {message()}
      </div>
    );
  };
}

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { "call-type": "", "call-name": "", "error-type": "", "error-value": null },
    createBlockCallErrorComponent(),
  );
}

const BadParametersErrorMessage: Component<{ valueRaw: string }> = (props) => {
  const value = createMemo(() => parseBadParametersErrorValue(props.valueRaw));

  const ListParameters: Component<
    {
      // parameter list
      l: string[];
      // reason
      r: string;
      // is_verbatim
      v: boolean;
    }
  > = (props) => {
    return (
      <Index each={props.l}>
        {(item) => {
          return (
            <li>
              {`${props.r}${props.v ? "逐字" : ""}参数「`}
              <Name name={(props.v ? "`" : "") + item()} />
              {`」`}
            </li>
          );
        }}
      </Index>
    );
  };

  return (
    <>
      <p>以下参数存在问题：</p>
      <ul class="list-disc list-inside">
        <ListParameters l={value().normal.missing} r="缺少必要" v={false} />
        <ListParameters l={value().verbatim.missing} r="缺少必要" v={true} />
        <ListParameters l={value().normal.unknown} r="未知" v={false} />
        <ListParameters l={value().verbatim.unknown} r="未知" v={true} />
        <ListParameters l={value().normal.duplicated} r="重复" v={false} />
        <ListParameters l={value().verbatim.duplicated} r="重复" v={true} />
      </ul>
    </>
  );

  function parseBadParametersErrorValue(value: string) {
    function parsePart(part: string) {
      const bad = {
        missing: [] as string[],
        unknown: [] as string[],
        duplicated: [] as string[],
      };
      for (const item of part.split(",")) {
        switch (item[0]) {
          case "!":
            bad.missing.push(item.slice(1));
            break;
          case "?":
            bad.unknown.push(item.slice(1));
            break;
          case "=":
            bad.duplicated.push(item.slice(1));
            break;
        }
      }
      return bad;
    }

    const [partNormal, partVerbatim] = value.split(";");

    return {
      normal: parsePart(partNormal!),
      verbatim: parsePart(partVerbatim!),
    };
  }
};

const Name: Component<{ name: string }> = (props) => {
  return <span class="text-gray-300">{props.name}</span>;
};
