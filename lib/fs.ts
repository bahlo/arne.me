import { join } from "node:path";

export function buildAbsolutePath(...segments: string[]): string {
  const cwd = process.cwd();
  const root = cwd.replace(/\.next\/server\/app(\/[a-z.\[\]]+)*(\/rsc)?/, "");
  return join(root, ...segments);
}
