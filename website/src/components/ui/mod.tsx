import {
  Component,
  createEffect,
  createSignal,
  JSX,
  on,
  onMount,
  Show,
} from "solid-js";
import { HiOutlineXCircle, HiSolidChevronDown } from "solid-icons/hi";

export { default as Loading } from "./Loading";

export const Card: Component<
  { children: JSX.Element; class?: string; bodyClass?: string }
> = (
  props,
) => {
  return (
    <div class={`card bg-base-100 shadow-xl ${props.class ?? ""}`}>
      <div class={`card-body h-full ${props.bodyClass ?? ""}`}>
        {props.children}
      </div>
    </div>
  );
};

export const BadgeBar: Component<{ children: JSX.Element; class?: string }> = (
  props,
) => {
  return (
    <div class={`flex justify-center gap-4 ${props.class ?? ""}`}>
      {props.children}
    </div>
  );
};

export const Badge: Component<{ children: JSX.Element }> = (props) => {
  return <div class="badge badge-neutral">{props.children}</div>;
};

export const Tabs: Component<{ children: JSX.Element; class?: string }> = (
  props,
) => {
  return <div class={`tabs ${props.class ?? ""}`}>{props.children}</div>;
};

export const Tab: Component<
  {
    children: JSX.Element;
    isActive?: boolean;
    onClick?: (ev: MouseEvent) => void;
  }
> = (
  props,
) => {
  const classes = () =>
    [
      props.isActive ? "tab-active" : "",
      props.onClick ? "" : "cursor-auto",
    ].join(" ");

  return (
    <div
      class={`tab tab-bordered ${classes()}`}
      onClick={props.onClick}
    >
      {props.children}
    </div>
  );
};

export const TabWithDropdown: Component<
  {
    summary: JSX.Element;
    isActive?: boolean;
    children: JSX.Element;
    onClick?: () => void;
  }
> = (props) => {
  const [_labelEl, setLabelEl] = createSignal<HTMLElement>();

  return (
    <Tab
      isActive={props.isActive}
      onClick={props.isActive ? props.onClick : undefined}
    >
      <Dropdown
        setLabelRef={setLabelEl}
        summary={props.summary}
        labelAsButton={false}
        labelClass="text-xs cursor-pointer"
        disabled={!props.isActive}
      >
        {props.children}
      </Dropdown>
    </Tab>
  );
};

export const Alert: Component<
  {
    type: "error";
    title: string;
    message: string;
    details?: string;
  }
> = (
  props,
) => {
  const typeClass = (): "alert-error" => {
    return `alert-${props.type}`;
  };

  return (
    // FIXME: 想要实现的是：横向可以有滚动条，纵向总是随着内容而增高，但不知道要怎么做。
    //        没有两处 `overflow-xauto`，高度正常，但过长的内容会溢出；
    //        有那两处的话，横向滚动正常，但纵向也变成滚动的了。
    <div class={`alert overflow-x-auto ${typeClass()}`}>
      <div class="flex flex-col gap-1 px-0 mx-0">
        <span class="flex items-center gap-4">
          <HiOutlineXCircle size={24} />
          <span class="font-bold">{props.title}</span>
        </span>
        <pre class="overflow-x-auto">
        <code>
          {props.message}
          <Show when={props.details}>
          <hr />
          {props.details}
          </Show>
        </code>
        </pre>
      </div>
    </div>
  );
};

export const Dropdown: Component<
  {
    setLabelRef?: (v: HTMLLabelElement) => void;
    summary: JSX.Element;
    children: JSX.Element;
    labelAsButton?: boolean;
    labelClass?: string;
    contentClass?: string;
    disabled?: boolean;
  }
> = (props) => {
  props.labelAsButton ??= true;
  props.disabled ??= false;

  let labelEl!: HTMLLabelElement;
  let ulEl!: HTMLUListElement;

  let isOpen = false;

  createEffect(on([() => props.disabled], () => {
    if (props.disabled) {
      isOpen = false;
    }
  }));

  const labelButtonClass = () =>
    props.labelAsButton ? "btn" : "inline-flex gap-1 cursor-pointer";

  onMount(() => {
    window.addEventListener("click", (ev) => {
      if (props.disabled) return;

      if (!isOpen) {
        if (ev.target === labelEl) {
          isOpen = true;
        }
        return;
      }
      labelEl.blur();
      ulEl.blur();
      isOpen = false;
    }, { capture: true });

    props.setLabelRef?.(labelEl);
  });

  return (
    <div class="dropdown">
      <label
        ref={labelEl}
        tabindex={props.disabled ? undefined : "0"}
        class={`${labelButtonClass()} ${props.labelClass ?? ""}`}
      >
        {props.summary}
        <HiSolidChevronDown
          style={{ "visibility": props.disabled ? "hidden" : undefined }}
        />
      </label>
      <ul
        ref={ulEl}
        tabindex={props.disabled ? undefined : "0"}
        class={`dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box ${
          props.contentClass ?? ""
        }`}
      >
        {props.children}
      </ul>
    </div>
  );
};

export const DropdownItem: Component<{ children: JSX.Element }> = (props) => {
  return <li>{props.children}</li>;
};

export const Button: Component<
  {
    children?: JSX.Element;
    type?: "neutral" | "primary" | "ghost";
    size?: "sm" | "xs";
    hasOutline?: boolean;
    active?: boolean;
    class?: string;
    onClick?: () => void;
  }
> = (props) => {
  const classes = () =>
    [
      (props.type ? `btn-${props.type}` : "") satisfies
        | ""
        | "btn-neutral"
        | "btn-primary"
        | "btn-ghost",
      (props.size ? `btn-${props.size}` : "") satisfies
        | ""
        | "btn-sm"
        | "btn-xs",
      props.hasOutline ? "btn-outline" : "",
      props.active ? "btn-active" : "",
      props.class ?? "",
    ].join(" ");

  return (
    <div
      class={`btn ${classes()}`}
      onClick={props.onClick}
    >
      {props.children}
    </div>
  );
};

export const Radio: Component<{ checked?: boolean }> = (props) => {
  return (
    <input type="radio" name="radio-1" class="radio" checked={props.checked} />
  );
};
