import { useState } from "react";
import { isTauri } from "../tauriEnv";

export function UpdateCheck() {
  const [status, setStatus] = useState("");

  if (!isTauri()) {
    return null;
  }

  async function checkForUpdates() {
    setStatus("جاري التحقق...");
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const { relaunch } = await import("@tauri-apps/plugin-process");
      const update = await check();
      if (!update) {
        setStatus("أنت تستخدم أحدث إصدار");
        return;
      }
      const yes = window.confirm(
        `يتوفر إصدار ${update.version}. هل تريد التحديث الآن؟`,
      );
      if (!yes) {
        setStatus("");
        return;
      }
      setStatus("جاري التحميل...");
      await update.downloadAndInstall();
      setStatus("جاري إعادة التشغيل...");
      await relaunch();
    } catch (e) {
      setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="update-check">
      <button type="button" className="secondary" onClick={checkForUpdates}>
        التحقق من التحديث
      </button>
      {status && <span className="muted update-status">{status}</span>}
    </div>
  );
}
