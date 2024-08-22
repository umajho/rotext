export {
  type DicexpEvaluation,
  type DicexpEvaluatorProvider,
  getDefaultStyleProviders as getDefaultDicexpStyleProviders,
  type Properties as AnkorWidgetDicexpProperties,
  registerCustomElement as registerCustomElementForAnkorWidgetDicexp,
} from "./Dicexp/mod";

export {
  getDefaultStyleProviders as getDefaultRefLinkStyleProviders,
  type Properties as AnkorWidgetRefLinkProperties,
  type RefAddress,
  type RefContentRenderer,
  registerCustomElement as registerCustomElementForAnkorWidgetRefLink,
} from "./RefLink/mod";
