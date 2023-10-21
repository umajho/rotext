export {
  type DicexpEvaluation,
  type DicexpEvaluatorProvider,
  getDefaultStyleProviders as getDefaultDicexpStyleProviders,
  type Properties as RoWidgetDicexpProperties,
  registerCustomElement as registerCustomElementForRoWidgetDicexp,
} from "./Dicexp/mod";

export {
  getDefaultStyleProviders as getDefaultRefLinkStyleProviders,
  type Properties as RoWidgetRefLinkProperties,
  type RefAddress,
  type RefContentRenderer,
  registerCustomElement as registerCustomElementForRoWidgetRefLink,
} from "./RefLink/mod";
