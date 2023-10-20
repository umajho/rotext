import { Component } from "solid-js";

import { createStepsRepresentationComponent } from "./steps-representation";

export type Loading = Component;
export type ErrorAlert = Component<{ error: Error; showsStack: boolean }>;
export type StepsRepresentation = ReturnType<
  typeof createStepsRepresentationComponent
>;
