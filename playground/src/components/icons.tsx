/**
 * 取自 <https://github.com/x64Bits/solid-icons>，许可协议随该项目。
 */
export {};

/**
 * 由于使用 solid-icons 库时无法设置颜色，手动将其放到这里。
 */
export default function FaSolidDice(props: { color?: string; class?: string }) {
  return (
    <svg
      fill={props.color ?? "currentColor"}
      class={props.class}
      stroke-width="0"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 640 512"
      height="1em"
      width="1em"
      style="overflow: visible;"
    >
      <path d="M274.9 34.3c-28.1-28.1-73.7-28.1-101.8 0L34.3 173.1c-28.1 28.1-28.1 73.7 0 101.8l138.8 138.8c28.1 28.1 73.7 28.1 101.8 0l138.8-138.8c28.1-28.1 28.1-73.7 0-101.8L274.9 34.3zM200 224a24 24 0 1 1 48 0 24 24 0 1 1-48 0zM96 200a24 24 0 1 1 0 48 24 24 0 1 1 0-48zm128 176a24 24 0 1 1 0-48 24 24 0 1 1 0 48zm128-176a24 24 0 1 1 0 48 24 24 0 1 1 0-48zm-128-80a24 24 0 1 1 0-48 24 24 0 1 1 0 48zm96 328c0 35.3 28.7 64 64 64h192c35.3 0 64-28.7 64-64V256c0-35.3-28.7-64-64-64H461.7c11.6 36 3.1 77-25.4 105.5L320 413.8V448zm160-120a24 24 0 1 1 0 48 24 24 0 1 1 0-48z">
      </path>
    </svg>
  );
}
