import { useEffect, useState } from "react";
import {
  createStaffRecord,
  getStaff,
  updateStaffRecord,
  workCenterSuggestions,
} from "../api";
import type { StaffInput } from "../types";

const emptyInput: StaffInput = {
  work_center: "",
  name: "",
  file_number: "",
  civil_id: "",
  job_title: "",
  hire_date: null,
  grade: "",
  email: "",
  phone: "",
  qualification: "",
};

interface Props {
  staffId: number | null;
  onSaved: () => void;
  onCancel: () => void;
}

export function StaffFormPage({ staffId, onSaved, onCancel }: Props) {
  const [input, setInput] = useState<StaffInput>(emptyInput);
  const [centers, setCenters] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    workCenterSuggestions()
      .then(setCenters)
      .catch(() => setCenters([]));
  }, []);

  useEffect(() => {
    if (staffId === null) {
      setInput(emptyInput);
      return;
    }
    setLoading(true);
    getStaff(staffId)
      .then((s) => {
        setInput({
          work_center: s.work_center,
          name: s.name,
          file_number: s.file_number,
          civil_id: s.civil_id,
          job_title: s.job_title,
          hire_date: s.hire_date,
          grade: s.grade,
          email: s.email,
          phone: s.phone,
          qualification: s.qualification,
        });
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [staffId]);

  function setField<K extends keyof StaffInput>(key: K, value: StaffInput[K]) {
    setInput((prev) => ({ ...prev, [key]: value }));
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const payload = {
        ...input,
        hire_date: input.hire_date?.trim() ? input.hire_date.trim() : null,
      };
      if (staffId === null) {
        await createStaffRecord(payload);
      } else {
        await updateStaffRecord(staffId, payload);
      }
      onSaved();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="page">
      <h2>{staffId === null ? "إضافة موظف" : "تعديل موظف"}</h2>
      <form className="staff-form panel" onSubmit={submit}>
        <label>
          مركز العمل
          <input
            list="work-centers"
            value={input.work_center}
            onChange={(e) => setField("work_center", e.target.value)}
          />
          <datalist id="work-centers">
            {centers.map((c) => (
              <option key={c} value={c} />
            ))}
          </datalist>
        </label>
        <label>
          الاسم *
          <input
            value={input.name}
            onChange={(e) => setField("name", e.target.value)}
            required
          />
        </label>
        <label>
          رقم الملف
          <input
            value={input.file_number}
            onChange={(e) => setField("file_number", e.target.value)}
          />
        </label>
        <label>
          الرقم المدني *
          <input
            value={input.civil_id}
            onChange={(e) => setField("civil_id", e.target.value)}
            required
          />
        </label>
        <label>
          المسمي الوظيفي
          <input
            value={input.job_title}
            onChange={(e) => setField("job_title", e.target.value)}
          />
        </label>
        <label>
          تاريخ التعيين
          <input
            type="date"
            value={input.hire_date ?? ""}
            onChange={(e) =>
              setField("hire_date", e.target.value ? e.target.value : null)
            }
          />
        </label>
        <label>
          الدرجة
          <input
            value={input.grade}
            onChange={(e) => setField("grade", e.target.value)}
          />
        </label>
        <label>
          البريد الإلكتروني الوزاري
          <input
            type="email"
            value={input.email}
            onChange={(e) => setField("email", e.target.value)}
          />
        </label>
        <label>
          هاتف
          <input
            value={input.phone}
            onChange={(e) => setField("phone", e.target.value)}
          />
        </label>
        <label className="full-width">
          المؤهل العلمي
          <input
            value={input.qualification}
            onChange={(e) => setField("qualification", e.target.value)}
          />
        </label>
        {error && <p className="error full-width">{error}</p>}
        <div className="form-actions full-width">
          <button type="button" className="secondary" onClick={onCancel}>
            إلغاء
          </button>
          <button type="submit" disabled={loading}>
            {loading ? "جاري الحفظ..." : "حفظ"}
          </button>
        </div>
      </form>
    </div>
  );
}
