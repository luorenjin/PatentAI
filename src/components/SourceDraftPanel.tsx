interface SourceDraftPanelProps {
  value: string;
  currentSessionDraft: string | null;
  onChange: (value: string) => void;
}

export function SourceDraftPanel({
  value,
  currentSessionDraft,
  onChange,
}: SourceDraftPanelProps) {
  const currentDraft = currentSessionDraft?.trim() ?? "";
  const pendingDraft = value.trim();
  const draftLength = value.trim().length;

  const draftStatus = currentSessionDraft
    ? currentDraft === pendingDraft
      ? "当前会话已使用这份草稿"
      : "草稿已修改，点击“重新载入演示”后生效"
    : "首次启动时会使用这份草稿发起分析";

  return (
    <section className="supportCard supportCard-draft">
      <div className="supportCardHeader">
        <div>
          <p className="panelEyebrow">原始输入</p>
          <h3>交底草稿 / 文档摘录</h3>
        </div>
        <span className="metricCard">{draftLength} 字</span>
      </div>

      <p className="supportCardDescription">
        实时分析会把这里的内容作为原始工程草稿送给后端。你可以粘贴真实交底初稿、会议纪要摘录或 PDF 提取文本。
      </p>

      <label className="controlLabel" htmlFor="source-draft-input">
        待分析草稿
      </label>
      <textarea
        id="source-draft-input"
        className="sourceDraftInput"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder="请输入待分析的技术交底草稿..."
      />

      <p className="sourceDraftHint">{draftStatus}</p>
    </section>
  );
}