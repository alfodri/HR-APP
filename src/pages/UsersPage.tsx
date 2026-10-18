import { useEffect, useState } from "react";
import { createHrUser, listHrUsers } from "../api";
import type { UserInfo } from "../types";

export function UsersPage() {
  const [users, setUsers] = useState<UserInfo[]>([]);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");

  async function load() {
    setUsers(await listHrUsers());
  }

  useEffect(() => {
    load().catch((e) => setError(String(e)));
  }, []);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setMessage("");
    try {
      await createHrUser(username, password);
      setUsername("");
      setPassword("");
      setMessage("تم إنشاء الحساب");
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="page">
      <h2>حسابات المستخدمين</h2>
      <div className="two-col">
        <section className="panel">
          <h3>إضافة حساب موارد بشرية</h3>
          <form onSubmit={submit} className="stack-form">
            <label>
              اسم المستخدم
              <input
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
              />
            </label>
            <label>
              كلمة المرور
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                minLength={6}
                required
              />
            </label>
            {error && <p className="error">{error}</p>}
            {message && <p className="success">{message}</p>}
            <button type="submit">إضافة</button>
          </form>
        </section>
        <section className="panel">
          <h3>الحسابات الحالية</h3>
          <table>
            <thead>
              <tr>
                <th>المستخدم</th>
                <th>الدور</th>
              </tr>
            </thead>
            <tbody>
              {users.map((u) => (
                <tr key={u.id}>
                  <td>{u.username}</td>
                  <td>موارد بشرية</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      </div>
    </div>
  );
}
