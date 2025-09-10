import fs from "node:fs/promises";
import path from "node:path";

import z from "zod";
import YAML from "yaml";

import { Navigation } from "../../../src/resource-managers/wiki";

import { buildNavigation as parseNavigation } from "./navigation";
import { DiscoveredNamespace, discoverPagesInNamespace } from "./discovery";
import { buildPage, Page } from "./pages";

export interface Namespace {
  configuration: NamespaceConfiguration;
  navigation: Navigation;
  pages: Record<string, Page>;
}

export type NamespaceMap =
  & Map<string, Namespace>
  & { readonly __tag: unique symbol };

const validFileNamesInNamespaceSpecialFolder = new Set([
  "configuration.yaml",
  "navigation.rotext",
]);

export async function collectNamespace(
  ns: DiscoveredNamespace,
): Promise<["ok", Namespace] | ["error", string]> {
  for (const name of await fs.readdir(path.join(ns.path, "$"))) {
    if (!validFileNamesInNamespaceSpecialFolder.has(name)) {
      return [
        "error",
        "unrecognized special file in namespace " +
        JSON.stringify(ns.namespace) + ": " + JSON.stringify(name),
      ];
    }
  }

  const cfgText = await fs.readFile(
    path.join(ns.path, "$", "configuration.yaml"),
    { encoding: "utf8" },
  );
  const cfgResult = parseConfiguration(cfgText);
  if (cfgResult[0] !== "ok") return cfgResult;
  const configuration = cfgResult[1];

  const navText = await fs.readFile(
    path.join(ns.path, "$", "navigation.rotext"),
    { encoding: "utf8" },
  );
  const navResult = parseNavigation(navText);
  if (navResult[0] !== "ok") return navResult;
  const navigation = navResult[1];

  const pages: Record<string, Page> = {};
  for (const { name, path } of await discoverPagesInNamespace(ns.path)) {
    const fullName = `${ns.namespace}:${name}`;
    const pageText = await fs.readFile(path, { encoding: "utf8" });
    const result = buildPage(pageText, { name, fullName });
    if (result[0] === "error") return result;
    pages[name] = result[1];
  }

  return ["ok", { configuration, navigation, pages }];
}

const namespaceConfigurationSchema = z.strictObject({
  namespace: z.strictObject({
    aliases: z.string().array().optional(),
  }),
});
type NamespaceConfiguration = z.infer<typeof namespaceConfigurationSchema>;

function parseConfiguration(
  text: string,
): ["ok", NamespaceConfiguration] | ["error", string] {
  const parsed = YAML.parse(text);
  const result = namespaceConfigurationSchema.safeParse(parsed);
  if (result.success) {
    return ["ok", result.data];
  } else {
    return ["error", result.error.message];
  }
}
