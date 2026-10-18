/** True when the UI runs inside the Tauri desktop shell. */
export function isTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__")
  );
}

export const DEV_BROWSER_HINT =
  "وضع التطوير: شغّل npm run dev (Vite + hr-dev على المنفذ 1421).";
