import { ComputedColor } from "@rotext/web-utils";

export function mixColor(
  colorA: ComputedColor,
  weightA: number,
  colorB: ComputedColor,
  weightB: number,
) {
  const mixedColor: ComputedColor = [0, 0, 0, null];
  for (let i = 0; i < 3; i++) {
    mixedColor[i] = (colorA[i]! * weightA + colorB[i]! * weightB) | 0;
  }
  return mixedColor;
}
