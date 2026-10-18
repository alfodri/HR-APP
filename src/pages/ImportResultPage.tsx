import type { ImportSummary } from "../types";

interface Props {
  summary: ImportSummary;
  onBack: () => void;
}

export function ImportResultPage({ summary, onBack }: Props) {
  return (
    <div className="page">
      <h2>نتيجة الاستيراد</h2>
      <div className="panel import-summary">
        <p><strong>إضافة:</strong> {summary.added}</p>
        <p><strong>تحديث:</strong> {summary.updated}</p>
        <p><strong>تخطي:</strong> {summary.skipped}</p>
        {summary.skip_messages.length > 0 && (
          <div>
            <h3>ملاحظات</h3>
            <ul>
              {summary.skip_messages.map((m) => (
                <li key={m}>{m}</li>
              ))}
            </ul>
          </div>
        )}
        <button type="button" onClick={onBack}>العودة للقائمة</button>
      </div>
    </div>
  );
}
