import { useEffect, useState } from "react";
import {
  getSession,
  login,
  logout,
  needsSetup,
  setupInitial,
} from "./api";
import { UpdateCheck } from "./components/UpdateCheck";
import { ImportResultPage } from "./pages/ImportResultPage";
import { LettersPage } from "./pages/LettersPage";
import { LoginPage } from "./pages/LoginPage";
import { SetupPage } from "./pages/SetupPage";
import { StaffFormPage } from "./pages/StaffFormPage";
import { StaffListPage } from "./pages/StaffListPage";
import { UsersPage } from "./pages/UsersPage";
import { isTauri } from "./tauriEnv";
import type { ImportSummary, UserInfo, View } from "./types";
import "./App.css";

type AppPhase = "loading" | "setup" | "login" | "app";

function App() {
  const [phase, setPhase] = useState<AppPhase>("loading");
  const [user, setUser] = useState<UserInfo | null>(null);
  const [view, setView] = useState<View>("staff");
  const [editStaffId, setEditStaffId] = useState<number | null>(null);
  const [importSummary, setImportSummary] = useState<ImportSummary | null>(null);

  useEffect(() => {
    (async () => {
      try {
        if (await needsSetup()) {
          setPhase("setup");
          return;
        }
        const session = await getSession();
        if (session) {
          setUser(session);
          setPhase("app");
        } else {
          setPhase("login");
        }
      } catch {
        setPhase("login");
      }
    })();
  }, []);

  useEffect(() => {
    if (!isTauri() || phase !== "app") {
      return;
    }
    (async () => {
      try {
        const { check } = await import("@tauri-apps/plugin-updater");
        const { relaunch } = await import("@tauri-apps/plugin-process");
        const update = await check();
        if (!update) {
          return;
        }
        const yes = window.confirm(
          `يتوفر تحديث ${update.version}. تثبيت الآن؟`,
        );
        if (!yes) {
          return;
        }
        await update.downloadAndInstall();
        await relaunch();
      } catch {
        /* offline or no release yet */
      }
    })();
  }, [phase]);

  async function handleLogout() {
    await logout();
    setUser(null);
    setView("staff");
    setPhase("login");
  }

  if (phase === "loading") {
    return <div className="center-screen muted">جاري التحميل...</div>;
  }

  if (phase === "setup") {
    return (
      <div className="center-screen">
        <SetupPage
          onSetup={setupInitial}
          onDone={(u) => {
            setUser(u);
            setPhase("app");
          }}
        />
      </div>
    );
  }

  if (phase === "login") {
    return (
      <div className="center-screen">
        <LoginPage
          doLogin={login}
          onLogin={(u) => {
            setUser(u);
            setPhase("app");
          }}
        />
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <h1>نظام الموارد البشرية</h1>
          <p className="muted">
            {user?.username} — موارد بشرية
          </p>
        </div>
        <nav className="main-nav">
          <button
            type="button"
            className={view === "staff" || view === "staff-form" ? "active" : ""}
            onClick={() => {
              setView("staff");
              setEditStaffId(null);
            }}
          >
            الموظفون
          </button>
          <button
            type="button"
            className={view === "letters" ? "active" : ""}
            onClick={() => setView("letters")}
          >
            طلبات الخطابات
          </button>
          <button
            type="button"
            className={view === "users" ? "active" : ""}
            onClick={() => setView("users")}
          >
            المستخدمون
          </button>
          <UpdateCheck />
          <button type="button" className="secondary" onClick={handleLogout}>
            خروج
          </button>
        </nav>
      </header>
      <main>
        {view === "staff" && (
          <StaffListPage
            onAdd={() => {
              setEditStaffId(null);
              setView("staff-form");
            }}
            onEdit={(id) => {
              setEditStaffId(id);
              setView("staff-form");
            }}
            onImportDone={(summary) => {
              setImportSummary(summary);
              setView("import-result");
            }}
          />
        )}
        {view === "staff-form" && (
          <StaffFormPage
            staffId={editStaffId}
            onSaved={() => {
              setEditStaffId(null);
              setView("staff");
            }}
            onCancel={() => {
              setEditStaffId(null);
              setView("staff");
            }}
          />
        )}
        {view === "import-result" && importSummary && (
          <ImportResultPage
            summary={importSummary}
            onBack={() => setView("staff")}
          />
        )}
        {view === "letters" && <LettersPage />}
        {view === "users" && <UsersPage />}
      </main>
    </div>
  );
}

export default App;
