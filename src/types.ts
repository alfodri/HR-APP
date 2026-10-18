export type Role = "hr";

export interface UserInfo {
  id: number;
  username: string;
  role: Role;
}

export interface Staff {
  id: number;
  work_center: string;
  name: string;
  file_number: string;
  civil_id: string;
  job_title: string;
  hire_date: string | null;
  grade: string;
  email: string;
  phone: string;
  qualification: string;
}

export interface StaffInput {
  work_center: string;
  name: string;
  file_number: string;
  civil_id: string;
  job_title: string;
  hire_date: string | null;
  grade: string;
  email: string;
  phone: string;
  qualification: string;
}

export interface ImportSummary {
  added: number;
  updated: number;
  skipped: number;
  skip_messages: string[];
}

export type View =
  | "staff"
  | "staff-form"
  | "users"
  | "letters"
  | "import-result";
