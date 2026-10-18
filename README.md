# HR App (نظام الموارد البشرية)

Desktop HR staff manager built with **Tauri 2** (Rust) and **React**. Data is stored in a local **SQLite** file (no separate database server).

Repository: [alfodri/HR-APP](https://github.com/alfodri/HR-APP)

## Features

- **HR-only** accounts: first launch creates an HR user; any signed-in HR user can add more HR accounts
- Staff list, search, add/edit, Excel import (same columns as `staff-new.xlsx`)
- In-app updates from **GitHub Releases** (Windows NSIS, Linux AppImage)
- Placeholder for future letter / PDF requests

## Development (frontend in browser)

For fast UI work in Cursor, use the browser at **http://localhost:1420/**:

```bash
npm install
npm run dev
```

This starts **Vite** (port 1420) and the **hr-dev** API (port 1421) with a separate database under `dev-data/`.

## Development (desktop window)

```bash
npm run dev:app
```

## Build installers

Set signing key path (generated once; private key is gitignored):

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat src-tauri/.tauri/hr-app.key)"
npm run tauri build
```

Artifacts: `src-tauri/target/release/bundle/`

- **Linux**: `.deb` for first install; **AppImage** for in-app updates
- **Windows**: build on Windows for NSIS installer + updates

### Publish a release on GitHub

1. Bump `version` in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) and [`package.json`](package.json).
2. Build on Ubuntu (AppImage) and Windows (NSIS) with `TAURI_SIGNING_PRIVATE_KEY` set.
3. Create a GitHub Release on **alfodri/HR-APP** and upload:
   - Installers and `.sig` files from the bundle folder
   - `latest.json` from the updater artifacts (Tauri generates this when `createUpdaterArtifacts` is true)

Installed apps check:  
`https://github.com/alfodri/HR-APP/releases/latest/download/latest.json`

Staff data in app data is kept when updating.

## Tests

```bash
cd src-tauri && cargo test
```
