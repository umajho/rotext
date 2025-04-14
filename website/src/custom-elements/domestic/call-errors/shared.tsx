import { Component, createMemo, Index, Match, Switch } from "solid-js";

export const ErrorMessage: Component<
  { errorType: string; errorValue: string | null }
> = (props) => {
  return (
    <Switch>
      <Match when={props.errorType === "TODO"}>
        <p>TODO: 实现渲染</p>
      </Match>
      <Match when={props.errorType === "UnknownCallee"}>
        <p>调用对象不存在。</p>
      </Match>
      <Match when={props.errorType === "BadParameters"}>
        <BadParametersErrorMessage valueRaw={props.errorValue!} />
      </Match>
      <Match when={true}>
        <Switch>
          <Match when={typeof props.errorValue === "string"}>
            <p>{`${props.errorType} (${props.errorValue})`}</p>
          </Match>
          <Match when={true}>
            <p>{props.errorType}</p>
          </Match>
        </Switch>
      </Match>
    </Switch>
  );
};

export const BadParametersErrorMessage: Component<{ valueRaw: string }> = (
  props,
) => {
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
        <ListParameters
          l={value().normal.missing}
          r="缺少必要"
          v={false}
        />
        <ListParameters
          l={value().verbatim.missing}
          r="缺少必要"
          v={true}
        />
        <ListParameters l={value().normal.unknown} r="未知" v={false} />
        <ListParameters
          l={value().verbatim.unknown}
          r="未知"
          v={true}
        />
        <ListParameters
          l={value().normal.duplicated}
          r="重复"
          v={false}
        />
        <ListParameters
          l={value().verbatim.duplicated}
          r="重复"
          v={true}
        />
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

export const Name: Component<{ name: string }> = (props) => {
  return <span class="text-gray-300">{props.name}</span>;
};

export function getCallTypeTitle(callType: "transclusion" | "extension" | "") {
  switch (callType) {
    case "transclusion":
      return "嵌入包含页面";
    case "extension":
      return "调用扩展";
    default:
      return "未知性质的调用";
  }
}
