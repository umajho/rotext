import { Component } from "solid-js";

import { createStepsRepresentationComponent } from "./steps-representation";

export type LoadingComponent = Component;
export type ErrorAlertComponent = Component<
  { error: Error; showsStack: boolean }
>;
export type StepsRepresentationComponent = ReturnType<
  typeof createStepsRepresentationComponent
>;
