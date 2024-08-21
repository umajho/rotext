export {
  createWidgetComponent as createRoWidgetComponent,
  type DisplayMode as RoWidgetDisplayMode,
  type LabelContentComponent as RoWidgetLabelContentComponent,
  type PopperContainerProperties as RoWidgetPopperContainerProperties,
  type PopperContentProperties as RoWidgetPopperContentProperties,
} from "./create-widget-component";

export {
  getWidgetOwner as getRoWidgetOwner,
  registerWidgetOwner as registerRoWidgetOwner,
  type WidgetOwner as RoWidgetOwner,
} from "./widget-owners-store";
