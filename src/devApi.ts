import type { ImportSummary, Staff, StaffInput, UserInfo } from "./types";

const DEV_API = "http://127.0.0.1:1421/api";
const TOKEN_KEY = "hr_dev_token";

function authHeaders(): HeadersInit {
  const token = sessionStorage.getItem(TOKEN_KEY);
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  return headers;
}

async function parseError(res: Response): Promise<string> {
  try {
    const data = await res.json();
    if (data && typeof data.error === "string") {
      return data.error;
    }
  } catch {
    /* ignore */
  }
  return res.statusText || "خطأ في الاتصال";
}

async function devGet<T>(path: string): Promise<T> {
  const res = await fetch(`${DEV_API}${path}`, { headers: authHeaders() });
  if (!res.ok) {
    throw new Error(await parseError(res));
  }
  if (res.status === 204) {
    return undefined as T;
  }
  return res.json() as Promise<T>;
}

async function devPost<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${DEV_API}${path}`, {
    method: "POST",
    headers: authHeaders(),
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    throw new Error(await parseError(res));
  }
  if (res.status === 204) {
    return undefined as T;
  }
  return res.json() as Promise<T>;
}

async function devPut<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${DEV_API}${path}`, {
    method: "PUT",
    headers: authHeaders(),
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new Error(await parseError(res));
  }
  return res.json() as Promise<T>;
}

async function devDelete(path: string): Promise<void> {
  const res = await fetch(`${DEV_API}${path}`, {
    method: "DELETE",
    headers: authHeaders(),
  });
  if (!res.ok) {
    throw new Error(await parseError(res));
  }
}

interface AuthResponse {
  user: UserInfo;
  token: string;
}

function storeAuth(res: AuthResponse): UserInfo {
  sessionStorage.setItem(TOKEN_KEY, res.token);
  return res.user;
}

export const devApi = {
  needsSetup: () => devGet<boolean>("/needs_setup"),
  setupInitial: async (username: string, password: string) =>
    storeAuth(await devPost<AuthResponse>("/setup", { username, password })),
  login: async (username: string, password: string) =>
    storeAuth(await devPost<AuthResponse>("/login", { username, password })),
  logout: async () => {
    await devPost("/logout");
    sessionStorage.removeItem(TOKEN_KEY);
  },
  getSession: () => devGet<UserInfo | null>("/session"),
  listHrUsers: () => devGet<UserInfo[]>("/users"),
  createHrUser: (username: string, password: string) =>
    devPost<UserInfo>("/users", { username, password }),
  listStaff: (query?: string) => {
    const q = query ? `?query=${encodeURIComponent(query)}` : "";
    return devGet<Staff[]>(`/staff${q}`);
  },
  getStaff: (id: number) => devGet<Staff>(`/staff/${id}`),
  createStaffRecord: (input: StaffInput) => devPost<Staff>("/staff", input),
  updateStaffRecord: (id: number, input: StaffInput) =>
    devPut<Staff>(`/staff/${id}`, input),
  deleteStaffRecord: (id: number) => devDelete(`/staff/${id}`),
  workCenterSuggestions: () => devGet<string[]>("/work_centers"),
  staffTotal: () => devGet<number>("/staff/total"),
  importStaffExcelFile: async (file: File): Promise<ImportSummary> => {
    const token = sessionStorage.getItem(TOKEN_KEY);
    const form = new FormData();
    form.append("file", file);
    const headers: Record<string, string> = {};
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }
    const res = await fetch(`${DEV_API}/import`, {
      method: "POST",
      headers,
      body: form,
    });
    if (!res.ok) {
      throw new Error(await parseError(res));
    }
    return res.json() as Promise<ImportSummary>;
  },
};
