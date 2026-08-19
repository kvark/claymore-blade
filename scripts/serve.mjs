#!/usr/bin/env node
/** Static file server for the wasm hunt. Binds 0.0.0.0:8080. */
import { createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve } from "node:path";

const root = resolve(process.cwd(), process.argv[2] ?? "dist/web");
const port = Number(process.env.PORT ?? 8080);
const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".wgsl": "text/plain; charset=utf-8",
};

const server = createServer((req, res) => {
  const url = new URL(req.url ?? "/", "http://127.0.0.1");
  let rel = decodeURIComponent(url.pathname);
  if (rel.endsWith("/")) rel += "index.html";
  const file = normalize(join(root, rel)).replace(/\\/g, "/");
  if (!file.startsWith(root.replace(/\\/g, "/"))) {
    res.writeHead(403);
    res.end("forbidden");
    return;
  }
  const path = existsSync(file) && statSync(file).isFile() ? file : join(root, "index.html");
  if (!existsSync(path)) {
    res.writeHead(404, { "content-type": "text/plain" });
    res.end("build the wasm hunt first: bash scripts/build-web.sh");
    return;
  }
  res.writeHead(200, { "content-type": mime[extname(path)] ?? "application/octet-stream" });
  createReadStream(path).pipe(res);
});

server.listen(port, "0.0.0.0", () => {
  console.log(`claymore ${root} -> 0.0.0.0:${port}`);
});
