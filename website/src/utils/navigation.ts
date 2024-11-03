import Global from "../global";
import { wikiResourceManager } from "../resource-managers/wiki";

/**
 * @param fullPageName 可能带有锚的完整页面名。
 */
export function navigateToWiki(fullPageName: string) {
  const authenticFullPageName = wikiResourceManager
    .getAuthenticFullPageName(fullPageName);

  Global.navigator!(`/wiki/${authenticFullPageName}`);
}
