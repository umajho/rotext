import { createSignal } from "solid-js";

export function createRotextExampleStore(opts: {
  originalInput: string;
  originalExpectedOutput: string;
  fixtureNames: string[] | null;
  fixtures: { [fixtureName: string]: string } | null;
}) {
  const [input, setInput] = createSignal(opts.originalInput);

  return {
    get input() {
      return input();
    },
    set input(v: string) {
      setInput(v);
    },
    get isInputOriginal() {
      return input() === opts.originalInput;
    },
    get originalExpectedOutput() {
      return opts.originalExpectedOutput;
    },
    reset() {
      setInput(opts.originalInput);
    },
    get fixtureNames() {
      return opts.fixtureNames;
    },
    get fixtures() {
      return opts.fixtures;
    },
  };
}

export type RotextExampleStore = ReturnType<typeof createRotextExampleStore>;
