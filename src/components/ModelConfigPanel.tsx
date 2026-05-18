import type { ModelProviderStatus } from "../types/patentSession";

interface ModelConfigPanelProps {
  status: ModelProviderStatus | null;
  isLoading: boolean;
  isRevealing: boolean;
  onRefresh: () => void;
  onReveal: () => void;
}

const statusLabelMap: Record<ModelProviderStatus["status"], string> = {
  ready: "实时分析已就绪",
  needs_configuration: "待补全配置",
  invalid_config: "配置格式异常",
  unsupported_provider: "供应商暂不支持",
  browser_mock: "浏览器预览",
};

export function ModelConfigPanel({
  status,
  isLoading,
  isRevealing,
  onRefresh,
  onReveal,
}: ModelConfigPanelProps) {
  const canReveal = Boolean(status?.configPath) && status?.status !== "browser_mock";
  const statusName = status?.status ?? "needs_configuration";

  return (
    <section className="supportCard supportCard-config">
      <div className="supportCardHeader">
        <div>
          <p className="panelEyebrow">模型配置</p>
          <h3>提供商 / 接口 / Key 校验</h3>
        </div>
        <span className={`configStatusBadge configStatusBadge-${statusName}`}>
          {status ? statusLabelMap[status.status] : "读取中"}
        </span>
      </div>

      <p className="supportCardDescription">
        {status?.message ?? "正在读取当前桌面端模型配置状态..."}
      </p>

      <dl className="configDetails">
        <div>
          <dt>Provider</dt>
          <dd>{status?.provider ?? "未配置"}</dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{status?.modelName ?? "未配置"}</dd>
        </div>
        <div>
          <dt>API URL</dt>
          <dd>{status?.apiUrl ?? "未配置"}</dd>
        </div>
        <div>
          <dt>API Key</dt>
          <dd>{status?.apiKeyPreview ?? "未配置"}</dd>
        </div>
        <div className="configDetails-path">
          <dt>配置文件</dt>
          <dd>{status?.configPath || "当前仅浏览器预览，不读取本地 Tauri 配置文件。"}</dd>
        </div>
      </dl>

      <div className="supportCardActions">
        <button
          type="button"
          className="ghostButton"
          onClick={onRefresh}
          disabled={isLoading}
        >
          {isLoading ? "校验中..." : "刷新校验"}
        </button>
        <button
          type="button"
          className="secondaryButton"
          onClick={onReveal}
          disabled={!canReveal || isLoading || isRevealing}
        >
          {isRevealing ? "打开中..." : "在文件夹中显示"}
        </button>
      </div>
    </section>
  );
}