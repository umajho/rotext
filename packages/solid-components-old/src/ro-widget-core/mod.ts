export {
  createWidgetComponent as createRoWidgetComponent,
  type DisplayMode as RoWidgetDisplayMode,
  type PrimeContentComponent as RoWidgetPrimeContentComponent,
  type WidgetContainerProperties as RoWidgetContainerProperties,
  type WidgetContentProperties as RoWidgetContentProperties,
} from "./create-widget-component";

export {
  getWidgetOwner as getRoWidgetOwner,
  registerWidgetOwner as registerRoWidgetOwner,
  type WidgetOwner as RoWidgetOwner,
} from "./widget-owners-store";
