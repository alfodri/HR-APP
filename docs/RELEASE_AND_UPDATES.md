# Releases and testing updates

## One-time: GitHub signing secret

The updater needs the **same** private key you use locally.

1. Open [HR-APP → Settings → Secrets and variables → Actions](https://github.com/alfodri/HR-APP/settings/secrets/actions).
2. **New repository secret**  
   - Name: `TAURI_SIGNING_PRIVATE_KEY`  
   - Value: entire contents of `src-tauri/.tauri/hr-app.key` (local only, never commit).

## Build Windows `.exe` + Linux AppImage (CI)

Windows installers **cannot** be built on Ubuntu. Use GitHub Actions:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Or: **Actions → Release → Run workflow**.

Download from the [Releases](https://github.com/alfodri/HR-APP/releases) page:

- **Windows**: `HR App_*_x64-setup.exe` (share this with staff)
- **Linux**: `HR App_*_amd64.AppImage` or `.deb`
- **Updater**: `latest.json` + `.sig` files (uploaded automatically)

## Test the update feature

1. Publish **v0.1.0** (tag + workflow above). Install `0.1.0` on a PC.
2. Bump version in `package.json` and `src-tauri/tauri.conf.json` to `0.1.1`, commit, push.
3. Tag and release **v0.1.1** the same way.
4. Open the **0.1.0** app → **التحقق من التحديث** (or restart; it checks on startup). Confirm prompt for `0.1.1`, install, restart.

Updater endpoint (in app):  
`https://github.com/alfodri/HR-APP/releases/latest/download/latest.json`

**Note:** `releases/latest` always points at the newest GitHub Release, so keep only one “current” release or use version tags consistently.

## Local Linux build (no Windows exe)

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat src-tauri/.tauri/hr-app.key)"
npm run tauri build
```

Output: `src-tauri/target/release/bundle/appimage/HR App_*_amd64.AppImage`
