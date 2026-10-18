import { useCallback, useEffect, useRef, useState } from "react";
import {
  deleteStaffRecord,
  importStaffExcel,
  importStaffExcelFile,
  listStaff,
  staffTotal,
} from "../api";
import { isTauri } from "../tauriEnv";
import type { ImportSummary, Staff } from "../types";

interface Props {
  onAdd: () => void;
  onEdit: (id: number) => void;
  onImportDone: (summary: ImportSummary) => void;
}

export function StaffListPage({ onAdd, onEdit, onImportDone }: Props) {
  const [staff, setStaff] = useState<Staff[]>([]);
  const [query, setQuery] = useState("");
  const [total, setTotal] = useState(0);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const [rows, count] = await Promise.all([
        listStaff(query.trim() || undefined),
        staffTotal(),
      ]);
      setStaff(rows);
      setTotal(count);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => {
    const t = setTimeout(() => {
      load();
    }, 200);
    return () => clearTimeout(t);
  }, [load]);

  async function runImport(file: File | string) {
    setImporting(true);
    setError("");
    try {
      const summary =
        typeof file === "string"
          ? await importStaffExcel(file)
          : await importStaffExcelFile(file);
      onImportDone(summary);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setImporting(false);
    }
  }

  async function handleImport() {
    if (!isTauri()) {
      fileInputRef.current?.click();
      return;
    }
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
    });
    if (!selected || typeof selected !== "string") {
      return;
    }
    await runImport(selected);
  }

  async function onFilePicked(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (file) {
      await runImport(file);
    }
  }

  async function handleDelete(id: number, name: string) {
    if (!window.confirm(`حذف الموظف: ${name}؟`)) {
      return;
    }
    try {
      await deleteStaffRecord(id);
      await load();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="page staff-page">
      <div className="toolbar">
        <div>
          <h2>الموظفون</h2>
          <p className="muted">الإجمالي: {total}</p>
        </div>
        <div className="toolbar-actions">
          <input
            className="search"
            placeholder="بحث بالاسم، الرقم المدني، رقم الملف، مركز العمل..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <input
            ref={fileInputRef}
            type="file"
            accept=".xlsx,.xls"
            hidden
            onChange={onFilePicked}
          />
          <button type="button" className="secondary" onClick={handleImport} disabled={importing}>
            {importing ? "جاري الاستيراد..." : "استيراد Excel"}
          </button>
          <button type="button" onClick={onAdd}>إضافة موظف</button>
        </div>
      </div>
      {error && <p className="error">{error}</p>}
      {loading && staff.length === 0 ? (
        <p className="muted">جاري التحميل...</p>
      ) : staff.length === 0 ? (
        <div className="empty-state panel">
          <p>لا يوجد موظفون بعد.</p>
          <p className="muted">أضف موظفاً يدوياً أو استورد ملف Excel بنفس أعمدة staff-new.xlsx</p>
        </div>
      ) : (
        <div className="table-wrap panel">
          <table className="staff-table">
            <thead>
              <tr>
                <th>الاسم</th>
                <th>مركز العمل</th>
                <th>رقم الملف</th>
                <th>الرقم المدني</th>
                <th>المسمي الوظيفي</th>
                <th>تاريخ التعيين</th>
                <th>الدرجة</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {staff.map((s) => (
                <tr key={s.id}>
                  <td>{s.name}</td>
                  <td>{s.work_center}</td>
                  <td>{s.file_number}</td>
                  <td>{s.civil_id}</td>
                  <td>{s.job_title}</td>
                  <td>{s.hire_date ?? "—"}</td>
                  <td>{s.grade}</td>
                  <td className="row-actions">
                    <button type="button" className="link" onClick={() => onEdit(s.id)}>
                      تعديل
                    </button>
                    <button
                      type="button"
                      className="link danger"
                      onClick={() => handleDelete(s.id, s.name)}
                    >
                      حذف
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
