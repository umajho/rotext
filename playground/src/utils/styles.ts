import { JSX } from "solid-js/jsx-runtime";

type CSSKey = keyof JSX.CSSProperties;

function getComputedCSSValueFromRaw(key: CSSKey, rawValue: string) {
  return getComputedCSSValue(key, (el) => el.style[key] = rawValue);
}

export function getComputedCSSValueOfClass(key: CSSKey, klass: string) {
  return getComputedCSSValue(key, (el) => el.classList.add(klass));
}

function getComputedCSSValue(
  key: CSSKey,
  action: (el: HTMLElement) => void,
): string {
  const containerEl = document.createElement("div");
  containerEl.style.visibility = "hidden";
  containerEl.style.width = "0";
  containerEl.style.height = "0";
  containerEl.style.overflow = "hidden";

  const el = document.createElement("div");
  action(el);

  containerEl.appendChild(el);
  document.body.appendChild(containerEl);

  const value = getComputedStyle(el)[key];

  containerEl.remove();

  return value;
}

export function getSizeInPx(size: string) {
  return parseFloat(getComputedCSSValueFromRaw("width", size));
}

export type ComputedColor = [r: number, g: number, b: number, a: number | null];

export function getComputedColor(
  color: string,
  alreadyComputed = false,
): ComputedColor | null {
  const computedColor = alreadyComputed
    ? color
    : getComputedCSSValueFromRaw("color", color);
  return parseComputedColor(computedColor);
}

/**
 * NOTE: 来自 `getComputedStyle` 的颜色总是 rgb 或 rgba：
 *       https://stackoverflow.com/a/67006298
 */
function parseComputedColor(color: string): ComputedColor | null {
  const result =
    /^rgba?\([^\d]*(\d+)[^\d]*(\d+)[^\d]*(\d+)[^\d]*(?:(\d+)[^\d]*)?\)$/.exec(
      color,
    );
  if (!result) {
    console.warn(`unknown computed color: ${color}`);
    return null;
  }

  const rgbaText = result.slice(1);
  // @ts-ignore
  return rgbaText.map((n) => n !== undefined ? parseInt(n) : null);
}

export function computedColorToCSSValue(c: ComputedColor) {
  const [r, g, b, a] = c;
  if (a === null) {
    return `rgb(${r}, ${g}, ${b})`;
  }
  return `rgb(${r}, ${g}, ${b}, ${a})`;
}
