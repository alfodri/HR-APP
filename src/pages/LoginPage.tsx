import { useState } from "react";
import type { UserInfo } from "../types";

interface Props {
  onLogin: (user: UserInfo) => void;
  doLogin: (username: string, password: string) => Promise<UserInfo>;
}

export function LoginPage({ onLogin, doLogin }: Props) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const user = await doLogin(username, password);
      onLogin(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="auth-card">
      <h1>تسجيل الدخول</h1>
      <p className="muted">نظام الموارد البشرية</p>
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
            autoComplete="current-password"
            required
          />
        </label>
        {error && <p className="error">{error}</p>}
        <button type="submit" disabled={loading}>
          {loading ? "جاري الدخول..." : "دخول"}
        </button>
      </form>
    </div>
  );
}
