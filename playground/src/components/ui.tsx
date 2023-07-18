import { Component, JSX, onMount, Show } from "solid-js";
import { HiOutlineXCircle } from "solid-icons/hi";

export const Card: Component<{ children: JSX.Element; class?: string }> = (
  props,
) => {
  return (
    <div class={`card bg-base-100 shadow-xl ${props.class ?? ""}`}>
      <div class="card-body h-full">
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

export const Tab: Component<{ children: JSX.Element; isActive?: boolean }> = (
  props,
) => {
  return (
    <div class={`tab tab-bordered ${props.isActive ? "tab-active" : ""}`}>
      {props.children}
    </div>
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

export const Loading: Component = () => {
  return <span class="loading loading-spinner loading-lg"></span>;
};

export const Dropdown: Component<
  {
    summary: JSX.Element;
    children: JSX.Element;
    buttonClass?: string;
    contentClass?: string;
  }
> = (props) => {
  let labelEl: HTMLLabelElement;
  let ulEl: HTMLUListElement;

  onMount(() => {
    let isOpen = false;
    window.addEventListener("click", (ev) => {
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
  });

  return (
    <div class="dropdown">
      <label
        ref={labelEl}
        tabindex="0"
        class={`btn ${props.buttonClass ?? ""}`}
      >
        {props.summary}
      </label>
      <ul
        ref={ulEl}
        tabindex="0"
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

export const Radio: Component<{ checked?: boolean }> = (props) => {
  return (
    <input type="radio" name="radio-1" class="radio" checked={props.checked} />
  );
};