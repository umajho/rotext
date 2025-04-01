import fs from "node:fs/promises";
import path from "node:path";

export interface DiscoveredNamespace {
  namespace: string;
  path: string;
}

export async function discoverNamespaces(
  basePath: string,
): Promise<DiscoveredNamespace[]> {
  return (await fs.readdir(basePath, { withFileTypes: true }))
    .filter((e) => e.isDirectory)
    .map((e): DiscoveredNamespace => ({
      namespace: e.name,
      path: path.join(e.path, e.name),
    }));
}

export interface DiscoveredPage {
  name: string;
  path: string;
}

export async function discoverPagesInNamespace(
  namespacePath: string,
): Promise<DiscoveredPage[]> {
  return (await fs.readdir(namespacePath, { recursive: true }))
    .filter((p) => !p.startsWith("$") && path.extname(p) === ".rotext")
    .map((p) => ({
      name: /(.*)\.rotext$/.exec(p)![1]!,
      path: path.join(namespacePath, p),
    }));
}
