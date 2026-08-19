#!/bin/sh
set -eu
cd /workspace
if curl -sf -o /dev/null --max-time 2 http://127.0.0.1:8080/; then
  exit 0
fi
if [ ! -f dist/web/index.html ]; then
  mkdir -p dist/web
  printf '%s\n' '<!doctype html><meta charset="utf-8"><body style="background:#0b0a09;color:#c8c0b0;font:16px sans-serif;padding:3rem">Building Claymore…</body>' > dist/web/index.html
fi
node scripts/serve.mjs dist/web >>/tmp/app-startup.log 2>&1 &
