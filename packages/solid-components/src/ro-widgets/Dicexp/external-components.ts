import { Component } from "solid-js";

import { ErrorAlert } from "../../common/mod";
import { createStepsRepresentationComponent } from "./steps-representation";

export type LoadingComponent = Component;
export type ErrorAlertComponent = typeof ErrorAlert;
export type StepsRepresentationComponent = ReturnType<
  typeof createStepsRepresentationComponent
>;
