import { invoke } from "@tauri-apps/api/core";
import type { ImportSummary, Staff, StaffInput, UserInfo } from "./types";
import { devApi } from "./devApi";
import { isTauri } from "./tauriEnv";

export async function needsSetup(): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>("needs_setup");
  }
  return devApi.needsSetup();
}

export async function setupInitial(
  username: string,
  password: string,
): Promise<UserInfo> {
  if (isTauri()) {
    return invoke<UserInfo>("setup_initial", { username, password });
  }
  return devApi.setupInitial(username, password);
}

/** @deprecated use setupInitial */
export const setupAdmin = setupInitial;

export async function login(
  username: string,
  password: string,
): Promise<UserInfo> {
  if (isTauri()) {
    return invoke<UserInfo>("login", { username, password });
  }
  return devApi.login(username, password);
}

export async function logout(): Promise<void> {
  if (isTauri()) {
    return invoke("logout");
  }
  return devApi.logout();
}

export async function getSession(): Promise<UserInfo | null> {
  if (isTauri()) {
    return invoke<UserInfo | null>("get_session");
  }
  return devApi.getSession();
}

export async function listHrUsers(): Promise<UserInfo[]> {
  if (isTauri()) {
    return invoke<UserInfo[]>("list_hr_users");
  }
  return devApi.listHrUsers();
}

export async function createHrUser(
  username: string,
  password: string,
): Promise<UserInfo> {
  if (isTauri()) {
    return invoke<UserInfo>("create_hr_user", { username, password });
  }
  return devApi.createHrUser(username, password);
}

export async function listStaff(query?: string): Promise<Staff[]> {
  if (isTauri()) {
    return invoke<Staff[]>("list_staff", { query: query || null });
  }
  return devApi.listStaff(query);
}

export async function getStaff(id: number): Promise<Staff> {
  if (isTauri()) {
    return invoke<Staff>("get_staff", { id });
  }
  return devApi.getStaff(id);
}

export async function createStaffRecord(input: StaffInput): Promise<Staff> {
  if (isTauri()) {
    return invoke<Staff>("create_staff_record", { input });
  }
  return devApi.createStaffRecord(input);
}

export async function updateStaffRecord(
  id: number,
  input: StaffInput,
): Promise<Staff> {
  if (isTauri()) {
    return invoke<Staff>("update_staff_record", { id, input });
  }
  return devApi.updateStaffRecord(id, input);
}

export async function deleteStaffRecord(id: number): Promise<void> {
  if (isTauri()) {
    return invoke("delete_staff_record", { id });
  }
  return devApi.deleteStaffRecord(id);
}

export async function workCenterSuggestions(): Promise<string[]> {
  if (isTauri()) {
    return invoke<string[]>("work_center_suggestions");
  }
  return devApi.workCenterSuggestions();
}

export async function staffTotal(): Promise<number> {
  if (isTauri()) {
    return invoke<number>("staff_total");
  }
  return devApi.staffTotal();
}

export async function importStaffExcel(path: string): Promise<ImportSummary> {
  return invoke<ImportSummary>("import_staff_excel", { path });
}

export async function importStaffExcelFile(
  file: File,
): Promise<ImportSummary> {
  if (isTauri()) {
    throw new Error("استخدم مسار الملف في تطبيق سطح المكتب");
  }
  return devApi.importStaffExcelFile(file);
}
