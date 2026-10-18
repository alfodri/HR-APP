#!/usr/bin/env bash
# Quick check that GitHub serves latest.json for the Tauri updater.
set -euo pipefail
URL="https://github.com/alfodri/HR-APP/releases/latest/download/latest.json"
echo "GET $URL"
code=$(curl -sS -o /tmp/hr-latest.json -w "%{http_code}" "$URL")
if [[ "$code" != "200" ]]; then
  echo "HTTP $code — no release with latest.json yet."
  echo "Publish a release: upload release-assets/latest.json + AppImage + .sig"
  exit 1
fi
python3 -m json.tool /tmp/hr-latest.json
echo "OK — updater URL is valid."
