# Releases and testing updates

## One-time: GitHub signing secret

The updater needs the **same** private key you use locally.

1. Open [HR-APP → Settings → Secrets and variables → Actions](https://github.com/alfodri/HR-APP/settings/secrets/actions).
2. **New repository secret**  
   - Name: `TAURI_SIGNING_PRIVATE_KEY`  
   - Value: entire contents of `src-tauri/.tauri/hr-app.key` (local only, never commit).

## Build Windows `.exe` + Linux AppImage (CI)

**Important:** `git push origin v0.1.x` only pushes a **tag**. The updater needs a **GitHub Release** with `latest.json` attached. Until CI finishes or you upload files manually, the app will show *Could not fetch a valid release JSON*.

1. Add secret **`TAURI_SIGNING_PRIVATE_KEY`** (see above). Without it, the Release workflow **fails** and nothing is published.
2. Tag and push (version in `package.json` / `tauri.conf.json` must match the tag):

```bash
git tag v0.1.1
git push origin v0.1.1
```

Or: **Actions → Release → Run workflow** (after fixing a failed run: **Re-run all jobs**).

If a tag failed, fix the secret then **Re-run** that workflow run—do not only push another tag unless you also intend to release that version.

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

## Error: “Could not fetch a valid release JSON from the remote”

The app is requesting `latest.json` from the **latest GitHub Release**. You see this when:

- There is **no published release** yet (only a git tag), or
- The release exists but **`latest.json` was not uploaded**, or
- The URL returns **404** (check in a browser).

**Quick fix (Linux, manual upload):**

```bash
./scripts/prepare-release-assets.sh
```

Then open [Create release v0.1.1](https://github.com/alfodri/HR-APP/releases/new?tag=v0.1.1), publish the release, and attach **all** of:

- `release-assets/latest.json` (required; name must be exactly `latest.json`)
- `release-assets/HR App_*_amd64.AppImage`
- `release-assets/HR App_*_amd64.AppImage.sig`

Verify: open  
`https://github.com/alfodri/HR-APP/releases/latest/download/latest.json`  
—you should see JSON with `"version": "0.1.1"`, not a 404 page.

If **Actions** builds fail, add the `TAURI_SIGNING_PRIVATE_KEY` secret first, then re-run the Release workflow or use the script above.

## Local Linux build (no Windows exe)

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat src-tauri/.tauri/hr-app.key)"
npm run tauri build
```

Output: `src-tauri/target/release/bundle/appimage/HR App_*_amd64.AppImage`
