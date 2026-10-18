# HR App — agent memory

Last updated from repo state. Keep this file accurate; max 500 lines.

## Purpose

Arabic RTL desktop **HR staff** app: **Tauri 2** (Rust) + **React** + **TypeScript**.

- Local **SQLite** only (`rusqlite` bundled). No network sync between PCs.
- Each install has its own DB. Coworkers get the **installer**, not shared staff data.
- Reference spreadsheet: `staff-new.xlsx` (228 rows, Arabic headers). App starts **empty** until import or manual add.
- UI language: Arabic, RTL. Rust command names stay English.

GitHub: **alfodri/HR-APP** (public).

## How to run

| Command | What it does |
|---------|----------------|
| `npm install` | Frontend deps |
| `npm run dev` | **Browser dev**: Vite `http://localhost:1420` + `hr-dev` API `127.0.0.1:1421` (via `concurrently`) |
| `npm run dev:vite` | Vite only (used by `tauri dev` as `beforeDevCommand`) |
| `npm run dev:api` | `cargo run --bin hr-dev --features dev-server` |
| `npm run dev:app` | Full **Tauri desktop** window (`tauri dev`) |
| `npm run build` | `tsc` + Vite production bundle |
| `npm run tauri build` | Release installers (set signing env; see Updates) |

**Browser vs desktop API**

- In the **Tauri window**, frontend uses `@tauri-apps/api` `invoke` → [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs).
- In the **browser** (`localhost:1420`), [`src/api.ts`](src/api.ts) calls [`src/devApi.ts`](src/devApi.ts) → HTTP on port **1421**. No `window.__TAURI_INTERNALS__` in Chrome.
- Dev DB path: `dev-data/hr_app.db` (gitignored). Production DB: OS app data dir, file `hr_app.db`.

## Accounts (HR only)

- **No admin role.** Single role: `hr` (serialized lowercase).
- **First launch**: setup screen → `setup_initial` creates first user as HR and logs in.
- **Any logged-in HR** can open **المستخدمون** and add more HR accounts (`create_hr_user`).
- Passwords: **Argon2**. Session in memory only; closing app or stopping dev server ends session.
- Migration: on DB open, `UPDATE users SET role = 'hr' WHERE role = 'admin'`.
- Dev HTTP auth: Bearer token in `sessionStorage` key `hr_dev_token` after login/setup.

## Staff data model

SQLite table `staff` (see [`src-tauri/src/db.rs`](src-tauri/src/db.rs)):

| Field (UI / Excel) | Notes |
|--------------------|--------|
| مركز العمل | `work_center` |
| الاسم | `name`, required |
| رقم الملف | `file_number` (text; may be empty or `جديد`) |
| الرقم المدني | `civil_id`, required, **unique** upsert key |
| المسمي الوظيفي | `job_title` |
| تاريخ التعيين | `hire_date` ISO `YYYY-MM-DD` optional |
| الدرجة | `grade` |
| البريد الإلكتروني الوزاري | `email` optional |
| هاتف | `phone` optional |
| المؤهل العلمي | `qualification` optional |

Excel column `م` is display order only; DB uses auto `id`.

**Import** ([`src-tauri/src/import.rs`](src-tauri/src/import.rs)): first sheet, Arabic headers, `calamine`. Hire dates: `dd-mm-yyyy`, `dd/mm/yyyy`, Excel serial (~20k–80k). Upsert by civil ID. Summary: added / updated / skipped.

Desktop import: native file dialog → path to `import_staff_excel`. Browser: multipart upload to `/api/import`.

## Frontend structure

- [`src/App.tsx`](src/App.tsx) — phases: loading, setup, login, app; views: staff, staff-form, users, letters, import-result.
- [`src/pages/`](src/pages/) — Setup, Login, StaffList, StaffForm, Users, Letters (placeholder), ImportResult.
- [`src/components/UpdateCheck.tsx`](src/components/UpdateCheck.tsx) — manual update check (Tauri only).
- [`src/tauriEnv.ts`](src/tauriEnv.ts) — `isTauri()` detector.
- [`index.html`](index.html) — `lang="ar"` `dir="rtl"`.

## Backend structure (Rust)

| File | Role |
|------|------|
| [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs) | Tauri commands, `AppState`, plugins (dialog, opener, updater, process) |
| [`src-tauri/src/auth.rs`](src-tauri/src/auth.rs) | Users, Argon2, `Role::Hr`, sessions |
| [`src-tauri/src/db.rs`](src-tauri/src/db.rs) | Migrations, `open_db` |
| [`src-tauri/src/staff.rs`](src-tauri/src/staff.rs) | CRUD, search, work-center suggestions |
| [`src-tauri/src/import.rs`](src-tauri/src/import.rs) | XLSX import + unit tests (`staff-new.xlsx`) |
| [`src-tauri/src/dev_server.rs`](src-tauri/src/dev_server.rs) | Axum API, feature `dev-server` only |
| [`src-tauri/src/bin/hr-dev.rs`](src-tauri/src/bin/hr-dev.rs) | Dev server binary |

Tauri commands (desktop): `needs_setup`, `setup_initial`, `login`, `logout`, `get_session`, `list_hr_users`, `create_hr_user`, `list_staff`, `get_staff`, `create_staff_record`, `update_staff_record`, `delete_staff_record`, `work_center_suggestions`, `staff_total`, `import_staff_excel`.

## Updates (GitHub Releases)

- Plugins: `tauri-plugin-updater`, `tauri-plugin-process`.
- Config: [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) — `createUpdaterArtifacts: true`, pubkey in `plugins.updater`, endpoint:
  `https://github.com/alfodri/HR-APP/releases/latest/download/latest.json`
- **Linux in-app update**: AppImage. **First install** can use `.deb`.
- **Windows**: NSIS on Windows build machine (not cross-compiled from Linux).
- Signing: private key at `src-tauri/.tauri/hr-app.key` (gitignored). Build with:
  `export TAURI_SIGNING_PRIVATE_KEY="$(cat src-tauri/.tauri/hr-app.key)"`
- App checks on startup (when logged into app) + button **التحقق من التحديث**. Staff SQLite in app data is **preserved** across updates.

## Tests

```bash
cd src-tauri && cargo test
```

Covers date parsing, empty DB + user flow, full import of `staff-new.xlsx` (228 rows).

## Not built yet

- **طلبات الخطابات**: placeholder page only. No PDF letter generation, no `letters` table.
- No central server; no multi-PC staff sync.

## Decisions (do not revert without user ask)

- SQLite embedded file, not external DB server.
- HR-only accounts; coworkers share installer, not data.
- Browser dev via `hr-dev` on 1421, not `invoke` in Chrome.
- Civil ID is the import upsert key; work-center strings stored as imported (no auto-merge of spelling variants).
- Plans in `.cursor/plans/` are historical; this memory file is the live summary for agents.

## Agent maintenance

When you change this project in ways listed in [`.cursor/rules/project-memory.mdc`](.cursor/rules/project-memory.mdc), rewrite this file before ending the chat. Stay under 500 lines.
