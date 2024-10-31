import {
  registerCustomElementForAnkorWidgetNavigation,
} from "@rotext/solid-components/internal";

import { BACKGROUND_COLOR, baseStyleProviders } from "./shared-thingy";

export function registerCustomElementForRefLink() {
  registerCustomElementForAnkorWidgetNavigation("ro-widget-ref-link", {
    baseStyleProviders,
    classes: {
      forLabelWrapper: "font-mono underline text-[#789922]",
      forNavigationAction: {
        enabled: "font-bold text-blue-500 hover:text-blue-700",
        disabled: "text-gray-600",
      },
    },
    backgroundColor: BACKGROUND_COLOR,
    label: ["text", (address) => `>>${address}`],
    innerPreviewRenderer: (rawAddrW, rendererOpts) => {
      rendererOpts.updateNavigationText(`>>${rawAddrW.currentValue}`);

      return {
        isAutoOpenable: true,
        render: (el, _renderOpts) => {
          el.innerText = JSON.stringify(rawAddrW.currentValue);
          el.style.color = "white";
          rawAddrW.onChange((value) => el.innerText = JSON.stringify(value));
        },
        navigate: () => {
          window.alert(`演示：请当作前往了 >>${rawAddrW.currentValue}。`);
        },
      };
    },
  });
}
