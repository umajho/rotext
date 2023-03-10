export interface V<VNode> {
  h: (_1: string, _2?: {}, _3?: string | VNode | (VNode | string)[]) => VNode;
  fragment: (nodes: VNode[]) => VNode;
}
