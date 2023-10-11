export const introduction: string = (await import(
  // @ts-ignore
  "./assets/introduction.rotext?raw"
)).default;
