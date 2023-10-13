import { Component } from "solid-js";
import { Dynamic } from "solid-js/web";

import type { Repr } from "dicexp";

export function createStepsRepresentationComponent(
  tagNameForStepsRepresentation: string,
): Component<{ repr: Repr }> {
  return (props) => {
    return (
      <Dynamic
        component={tagNameForStepsRepresentation}
        repr={props.repr}
      />
    );
  };
}
