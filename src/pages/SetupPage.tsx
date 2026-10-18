import { useState } from "react";
import { DEV_BROWSER_HINT } from "../tauriEnv";
import { isTauri } from "../tauriEnv";
import type { UserInfo } from "../types";

interface Props {
  onDone: (user: UserInfo) => void;
  onSetup: (username: string, password: string) => Promise<UserInfo>;
}

export function SetupPage({ onDone, onSetup }: Props) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (password !== confirm) {
      setError("كلمتا المرور غير متطابقتين");
      return;
    }
    setLoading(true);
    try {
      const user = await onSetup(username, password);
      onDone(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="auth-card">
      <h1>إعداد النظام</h1>
      <p className="muted">أنشئ أول حساب موارد بشرية</p>
      {!isTauri() && <p className="muted">{DEV_BROWSER_HINT}</p>}
      <form onSubmit={submit}>
        <label>
          اسم المستخدم
          <input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoComplete="username"
            required
          />
        </label>
        <label>
          كلمة المرور
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="new-password"
            minLength={6}
            required
          />
        </label>
        <label>
          تأكيد كلمة المرور
          <input
            type="password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            autoComplete="new-password"
            minLength={6}
            required
          />
        </label>
        {error && <p className="error">{error}</p>}
        <button type="submit" disabled={loading}>
          {loading ? "جاري الإنشاء..." : "إنشاء الحساب"}
        </button>
      </form>
    </div>
  );
}
