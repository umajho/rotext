import Global from "../global";
import { wikiResourceManager } from "../resource-managers/wiki";

/**
 * FIXME!!: 目前无论 `shouldOpenNewTab` 是什么，都会在当前页面进行跳转，因为我还
 * 没找到让 solid router 打开新标签页的办法。
 */
export function navigateToAddress(
  address: ["reference" | "internal", string],
  opts?: {
    heading?: string;

    shouldOpenNewTab: boolean;
  },
) {
  switch (address[0]) {
    case "reference": {
      let addrWithH = address[1];
      if (opts?.heading) {
        if (!/\d$/.test(addrWithH)) {
          // 对于包含串号的引用链接的地址，为防止混淆，若存在指向 heading 的锚，
          // 则应存在楼号。
          addrWithH += "#1";
        }
        addrWithH += opts.heading ? "#" + opts.heading : "";
      }
      navigateToPost(addrWithH);
      break;
    }
    case "internal": {
      const addrWithH = address[1] + (opts?.heading ? "#" + opts.heading : "");
      navigateToWiki(addrWithH);
      break;
    }
    default:
      console.warn(`不知要如何导航至 ${JSON.stringify(address)}`);
  }
}

export function navigateToPost(address: string) {
  window.alert(`演示：请当作前往了 >>${address}。`);
}

/**
 * @param fullPageName 可能带有锚的完整页面名。
 */
export function navigateToWiki(fullPageName: string) {
  const authenticFullPageName = wikiResourceManager
    .getAuthenticFullPageName(fullPageName);

  Global.navigator!(`/wiki/${authenticFullPageName}`);
}
