import { copyFileSync, existsSync, writeFileSync } from "node:fs";

const shell = "dist/client/_shell.html";
if (!existsSync(shell)) {
  console.error("pages-entry: missing", shell);
  process.exit(1);
}
copyFileSync(shell, "dist/client/index.html");
copyFileSync(shell, "dist/client/404.html");
writeFileSync("dist/client/.nojekyll", "");
