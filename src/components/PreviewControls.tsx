import type { PdfProcessingMode, SessionStage } from "../types/patentSession";

interface PreviewControlsProps {
  stage: SessionStage;
  pdfMode: PdfProcessingMode;
  zoom: number;
  agentView: boolean;
  showHighlights: boolean;
  annotationCount: number;
  validationIssueCount: number;
  includeAnswerHistory: boolean;
  canDownload: boolean;
  canTerminate: boolean;
  isBusy: boolean;
  onPdfModeChange: (mode: PdfProcessingMode) => void;
  onZoomChange: (nextZoom: number) => void;
  onAgentViewChange: (value: boolean) => void;
  onToggleHighlights: () => void;
  onIncludeAnswerHistoryChange: (value: boolean) => void;
  onDownload: () => void;
  onTerminate: () => void;
}

const stageLabels: Record<SessionStage, string> = {
  idle: "未开始",
  analyzing: "分析中",
  awaiting_answer: "等待回答",
  updating_preview: "更新预览",
  advisory_complete: "建议已完整",
  terminated: "已终止",
};

export function PreviewControls({
  stage,
  pdfMode,
  zoom,
  agentView,
  showHighlights,
  annotationCount,
  validationIssueCount,
  includeAnswerHistory,
  canDownload,
  canTerminate,
  isBusy,
  onPdfModeChange,
  onZoomChange,
  onAgentViewChange,
  onToggleHighlights,
  onIncludeAnswerHistoryChange,
  onDownload,
  onTerminate,
}: PreviewControlsProps) {
  return (
    <div className="previewControls">
      <div className="controlGroup">
        <span className="controlLabel">会话阶段</span>
        <span className="stageBadge">{stageLabels[stage]}</span>
      </div>
      <div className="controlGroup">
        <span className="controlLabel">PDF 处理方式</span>
        <div className="segmentedControl">
          <button
            type="button"
            className={pdfMode === "direct_to_model" ? "segment active" : "segment"}
            onClick={() => onPdfModeChange("direct_to_model")}
            disabled={isBusy}
          >
            直接提交大模型
          </button>
          <button
            type="button"
            className={pdfMode === "local_preprocess" ? "segment active" : "segment"}
            onClick={() => onPdfModeChange("local_preprocess")}
            disabled={isBusy}
          >
            本地预处理
          </button>
        </div>
      </div>
      <div className="controlGroup controlGroup-inline">
        <span className="controlLabel">当前摘要</span>
        <div className="toggleRow">
          <span className="metricCard">标注 {annotationCount}</span>
          <span className="metricCard">校验 {validationIssueCount}</span>
        </div>
      </div>
      <div className="controlGroup controlGroup-inline">
        <span className="controlLabel">视图控制</span>
        <div className="toggleRow">
          <label className="toggleControl">
            <input
              type="checkbox"
              checked={agentView}
              onChange={(event) => onAgentViewChange(event.currentTarget.checked)}
            />
            代理人视图
          </label>
          <button
            type="button"
            className="ghostButton"
            onClick={onToggleHighlights}
          >
            {showHighlights ? "移除所有高亮" : "恢复高亮"}
          </button>
        </div>
      </div>
      <div className="controlGroup controlGroup-inline">
        <span className="controlLabel">缩放</span>
        <div className="zoomRow">
          <button type="button" className="ghostButton" onClick={() => onZoomChange(zoom - 10)}>
            -
          </button>
          <span className="zoomValue">{zoom}%</span>
          <button type="button" className="ghostButton" onClick={() => onZoomChange(zoom + 10)}>
            +
          </button>
        </div>
      </div>
      <div className="controlGroup controlGroup-inline">
        <label className="toggleControl">
          <input
            type="checkbox"
            checked={includeAnswerHistory}
            onChange={(event) =>
              onIncludeAnswerHistoryChange(event.currentTarget.checked)
            }
          />
          下载时包含追问记录
        </label>
      </div>
      <div className="controlGroup actionButtons">
        <button
          type="button"
          className="secondaryButton"
          onClick={onTerminate}
          disabled={!canTerminate || isBusy}
        >
          提前终止
        </button>
        <button
          type="button"
          className="primaryButton"
          onClick={onDownload}
          disabled={!canDownload || isBusy}
        >
          下载当前版本
        </button>
      </div>
    </div>
  );
}