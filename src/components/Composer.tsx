import type { SessionStage } from "../types/patentSession";

interface ComposerProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  onSkip: () => void;
  canSubmit: boolean;
  canSkip: boolean;
  isBusy: boolean;
  stage: SessionStage;
}

const stageHints: Record<SessionStage, string> = {
  idle: "准备开始演示分析。",
  analyzing: "正在分析，请稍候。",
  awaiting_answer: "当前可以继续回答或跳过。",
  updating_preview: "正在刷新右侧预览。",
  advisory_complete: "AI 认为信息已较完整，你仍可继续补充。",
  terminated: "会话已终止，输入区只保留查看状态。",
};

export function Composer({
  value,
  onChange,
  onSubmit,
  onSkip,
  canSubmit,
  canSkip,
  isBusy,
  stage,
}: ComposerProps) {
  return (
    <div className="composerPanel">
      <label className="composerLabel" htmlFor="composerInput">
        回答 AI 追问
      </label>
      <textarea
        id="composerInput"
        className="composerInput"
        placeholder="补充现有技术差异、参数范围或控制条件..."
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
        disabled={isBusy || stage === "terminated"}
      />
      <div className="composerActions">
        <button
          type="button"
          className="primaryButton"
          onClick={onSubmit}
          disabled={!canSubmit || !value.trim()}
        >
          发送回答
        </button>
        <button
          type="button"
          className="secondaryButton"
          onClick={onSkip}
          disabled={!canSkip}
        >
          跳过当前追问
        </button>
      </div>
      <p className="composerHint">{stageHints[stage]}</p>
    </div>
  );
}