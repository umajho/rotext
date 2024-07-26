import { Component, createContext, JSX, useContext } from "solid-js";

import {
  createRotextProcessorsStore,
  RotextProcessorName,
  RotextProcessorsStore,
} from "../hooks/rotext-processors-store";

const RotextProcessorsStoreContext = createContext<RotextProcessorsStore>();

export const RotextProcessorsStoreProvider: Component<{
  children: JSX.Element;
  initialProcessorName: RotextProcessorName;
  onCurrentProcessorNameChange: (newName: RotextProcessorName) => void;
}> = (props) => {
  const store = createRotextProcessorsStore({
    initialProcessorName: props.initialProcessorName,
    onCurrentProcessorNameChange: props.onCurrentProcessorNameChange,
  });

  return (
    <RotextProcessorsStoreContext.Provider value={store}>
      {props.children}
    </RotextProcessorsStoreContext.Provider>
  );
};

export function useRotextProcessorsStore() {
  return useContext(RotextProcessorsStoreContext);
}
