import { Accessor, createEffect, createSignal, on } from "solid-js";

import { wikiResourceManager } from "../../../resource-managers/wiki";
import { Address, stringifyAddress } from "../../../utils/address";

export type PreviewContent =
  | ["ok", CutAndCloneContentForPreviewResult]
  /** 没有找到对应页面。 */
  | ["page_not_found"]
  /** 找到了对应页面，但没有对应标题。 */
  | ["heading_not_found"]
  /** 是正在进行编辑的内容，无法提供预览。 */
  | ["live"]
  /** 由于其他原因无法提供预览。 */
  | ["not_capable"]
  /** 输入变更。 */
  | ["input_changed"]
  | ["todo"];

export function createPreviewContent(
  address: Accessor<Address>,
): [Accessor<PreviewContent | null>, { reload: (address: Address) => void }] {
  let initialAddress = address();
  let currentAddressInString = stringifyAddress(initialAddress);
  const [content, setContent] = createSignal<PreviewContent | null>(null);

  createEffect(
    on([address], () => setContent(["input_changed"]), { defer: true }),
  );

  async function update(address: Address) {
    const addressInString = stringifyAddress(address);
    currentAddressInString = addressInString;
    const content = await getPreviewContent(address);
    if (currentAddressInString !== addressInString) return;
    setContent(content);
  }

  // 不 await。
  update(initialAddress);

  return [content, {
    reload: (address) => update(address),
  }];
}

async function getPreviewContent(
  address: Address,
): Promise<PreviewContent> {
  switch (address[0]) {
    case "reference/textual":
    case "reference/numeric":
      return ["todo"];
    case "internal": {
      const [_, fullName, anchor] = address;

      const page = await wikiResourceManager.getPage(fullName!);
      if (!page) return ["page_not_found"];
      const cutContent = cutAndCloneContentForPreview(page, anchor);
      if (!cutContent) return ["heading_not_found"];
      return ["ok", cutContent];
    }
    case "live":
      return ["live"];
    default:
      return ["not_capable"];
  }
}

interface CutAndCloneContentForPreviewResult {
  content: NodeListOf<ChildNode>;
  hasContentBefore: boolean;
  hasContentAfter: boolean;
}

function cutAndCloneContentForPreview(
  fullContent: DocumentFragment,
  heading: string | null,
): CutAndCloneContentForPreviewResult | null {
  let startNode: Node | null;
  if (heading) {
    startNode = [...fullContent.querySelectorAll("h1,h2,h3,h4,h5,h6")]
      .find((hEl) => hEl.textContent === heading) ?? null;
  } else {
    startNode = fullContent.firstElementChild;
  }
  if (!startNode) return null;
  let curNode: Node | null = startNode;

  const hasContentBefore = curNode.parentNode!.firstChild !== curNode;

  // 由于不明原因，使用 `template` 元素，并通过其 `.content` 获取到的
  // DocumentFragment 总是为空（Chrome、Safari 都是如此）。
  // 额外观察：在 Chrome Dev 控制台中将 log 出来的 `template` 元素存为临时变量，
  // 打印时显示其中有预期的内容，但无论 `.content` 还是 `.innerHTML` 中都没有内
  // 容，直接打 log 也是如此。
  // 退而使用 `div` 元素。
  const tEl = document.createElement("div");

  let shouldStopBeforeHeading = false;
  while (curNode) {
    if ("tagName" in curNode && /^H[1-6]$/.test((curNode as Element).tagName)) {
      if (shouldStopBeforeHeading) break;
    } else {
      shouldStopBeforeHeading = true;
    }
    tEl.append(curNode.cloneNode(true));
    curNode = curNode.nextSibling;
  }

  if (tEl.childElementCount || !heading) {
    const hasContentAfter = !!curNode;

    return {
      content: tEl.childNodes,
      hasContentBefore,
      hasContentAfter,
    };
  } else {
    return null;
  }
}
