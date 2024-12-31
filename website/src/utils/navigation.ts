import Global from "../global";
import { wikiResourceManager } from "../resource-managers/wiki";

import { Address, reconstructAddressAsText } from "./address";

/**
 * FIXME!!: 目前无论 `shouldOpenNewTab` 是什么，都会在当前页面进行跳转，因为我还
 * 没找到让 solid router 打开新标签页的办法。
 */
export function navigateToAddress(
  address: Address,
  _opts?: { shouldOpenNewTab: boolean },
) {
  switch (address[0]) {
    case "reference/textual":
    case "reference/numeric":
      window.alert(`演示：请当作前往了 ${reconstructAddressAsText(address)}。`);
      return;
    case "wiki": {
      const [_, fullName, anchor] = address;
      const addrWithH = fullName + (anchor ? `#${anchor}` : "");
      navigateToWiki(addrWithH);
      break;
    }
    default:
      console.warn(`不知要如何导航至 ${JSON.stringify(address)}`);
  }
}

/**
 * @param fullPageName 可能带有锚的完整页面名。
 */
function navigateToWiki(fullPageName: string) {
  const authenticFullPageName = wikiResourceManager
    .getAuthenticFullPageName(fullPageName);

  Global.navigator!(`/wiki/${authenticFullPageName}`);
}
