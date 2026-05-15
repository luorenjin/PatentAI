import { useEffect, useState } from "react";
import "./App.css";
import { AnnotationLegend } from "./components/AnnotationLegend";
import { Composer } from "./components/Composer";
import { ConversationTimeline } from "./components/ConversationTimeline";
import { ModelConfigPanel } from "./components/ModelConfigPanel";
import { PreviewControls } from "./components/PreviewControls";
import { PreviewPanel } from "./components/PreviewPanel";
import { SourceDraftPanel } from "./components/SourceDraftPanel";
import { ValidationSummary } from "./components/ValidationSummary";
import { defaultDemoDraft } from "./lib/defaultDemoDraft";
import {
  buildDemoSettings,
  downloadDemoSnapshot,
  getModelProviderStatus,
  revealModelProviderConfig,
  skipDemoQuestion,
  startDemoSession,
  submitDemoAnswer,
  terminateDemoSession,
  updateDemoSettings,
} from "./lib/demoSessionApi";
import type {
  DownloadResponse,
  ModelProviderStatus,
  PdfProcessingMode,
  SessionSnapshot,
} from "./types/patentSession";

type PendingAction =
  | "starting"
  | "submitting"
  | "skipping"
  | "settings"
  | "terminating"
  | "downloading"
  | null;

const pdfModeStorageKey = "patentmate.demo.pdfMode";
const sourceDraftStorageKey = "patentmate.demo.sourceDraft";

function readInitialPdfMode(): PdfProcessingMode {
  const storedMode = window.localStorage.getItem(pdfModeStorageKey);
  return storedMode === "local_preprocess"
    ? "local_preprocess"
    : "direct_to_model";
}

function readInitialSourceDraft() {
  const storedDraft = window.localStorage.getItem(sourceDraftStorageKey);
  return storedDraft && storedDraft.trim() ? storedDraft : defaultDemoDraft;
}

function buildConfigErrorStatus(message: string): ModelProviderStatus {
  return {
    status: "invalid_config",
    message,
    configPath: "",
    createdTemplate: false,
    provider: null,
    apiUrl: null,
    modelName: null,
    hasApiKey: false,
    apiKeyPreview: null,
  };
}

function App() {
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [pendingAction, setPendingAction] = useState<PendingAction>("starting");
  const [composerValue, setComposerValue] = useState("");
  const [sourceDraft, setSourceDraft] = useState(readInitialSourceDraft);
  const [pdfModePreference, setPdfModePreference] =
    useState<PdfProcessingMode>(readInitialPdfMode);
  const [showHighlights, setShowHighlights] = useState(true);
  const [agentView, setAgentView] = useState(false);
  const [includeAnswerHistory, setIncludeAnswerHistory] = useState(false);
  const [zoom, setZoom] = useState(100);
  const [downloadFeedback, setDownloadFeedback] =
    useState<DownloadResponse | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [modelProviderStatus, setModelProviderStatus] =
    useState<ModelProviderStatus | null>(null);
  const [isLoadingConfigStatus, setIsLoadingConfigStatus] = useState(false);
  const [isRevealingConfigPath, setIsRevealingConfigPath] = useState(false);

  useEffect(() => {
    void bootstrapSession(readInitialPdfMode(), readInitialSourceDraft());
    void refreshModelProviderStatus();
  }, []);

  useEffect(() => {
    window.localStorage.setItem(pdfModeStorageKey, pdfModePreference);
  }, [pdfModePreference]);

  useEffect(() => {
    window.localStorage.setItem(sourceDraftStorageKey, sourceDraft);
  }, [sourceDraft]);

  async function refreshModelProviderStatus() {
    setIsLoadingConfigStatus(true);

    try {
      const nextStatus = await getModelProviderStatus();
      setModelProviderStatus(nextStatus);
    } catch (error) {
      setModelProviderStatus(buildConfigErrorStatus(String(error)));
    } finally {
      setIsLoadingConfigStatus(false);
    }
  }

  async function bootstrapSession(
    pdfMode: PdfProcessingMode,
    nextSourceDraft: string,
  ) {
    const normalizedDraft = nextSourceDraft.trim();

    if (!normalizedDraft) {
      setErrorMessage("请先输入待分析的交底草稿，再启动会话。");
      setPendingAction(null);
      return;
    }

    setPendingAction("starting");
    setErrorMessage(null);
    setDownloadFeedback(null);
    setPdfModePreference(pdfMode);

    try {
      const snapshot = await startDemoSession({
        settings: buildDemoSettings(pdfMode),
        sourceDraft: normalizedDraft,
      });
      setSession(snapshot);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
      void refreshModelProviderStatus();
    }
  }

  async function handleRevealModelProviderConfig() {
    if (!modelProviderStatus?.configPath) {
      return;
    }

    setIsRevealingConfigPath(true);

    try {
      await revealModelProviderConfig(modelProviderStatus.configPath);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsRevealingConfigPath(false);
    }
  }

  async function handleSubmit() {
    if (!session || !composerValue.trim()) {
      return;
    }

    setPendingAction("submitting");
    setErrorMessage(null);
    setDownloadFeedback(null);

    try {
      const nextSession = await submitDemoAnswer({
        session,
        answer: composerValue.trim(),
      });
      setSession(nextSession);
      setComposerValue("");
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
    }
  }

  async function handleSkip() {
    if (!session) {
      return;
    }

    setPendingAction("skipping");
    setErrorMessage(null);
    setDownloadFeedback(null);

    try {
      const nextSession = await skipDemoQuestion({ session });
      setSession(nextSession);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
    }
  }

  async function handlePdfModeChange(nextMode: PdfProcessingMode) {
    window.localStorage.setItem(pdfModeStorageKey, nextMode);
    setPdfModePreference(nextMode);
    setDownloadFeedback(null);

    if (!session) {
      return;
    }

    setPendingAction("settings");
    setErrorMessage(null);

    try {
      const nextSession = await updateDemoSettings({
        session,
        settings: buildDemoSettings(nextMode),
      });
      setSession(nextSession);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
    }
  }

  async function handleTerminate() {
    if (!session) {
      return;
    }

    setPendingAction("terminating");
    setErrorMessage(null);
    setDownloadFeedback(null);

    try {
      const nextSession = await terminateDemoSession({ session });
      setSession(nextSession);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
    }
  }

  async function handleDownload() {
    if (!session) {
      return;
    }

    setPendingAction("downloading");
    setErrorMessage(null);

    try {
      const feedback = await downloadDemoSnapshot({
        session,
        includeAnswerHistory,
      });
      setDownloadFeedback(feedback);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setPendingAction(null);
    }
  }

  const isBusy = pendingAction !== null;
  const canSubmit = Boolean(session?.actions.canSubmit) && !isBusy;
  const canSkip = Boolean(session?.actions.canSkip) && !isBusy;
  const canDownload = Boolean(session?.actions.canDownload) && !isBusy;
  const canTerminate = Boolean(session?.actions.canTerminate) && !isBusy;
  const activePdfMode = session?.settings.pdfMode ?? pdfModePreference;
  const annotationCount = session?.disclosure.annotations.length ?? 0;
  const validationIssueCount = session?.validationIssues.length ?? 0;
  const downloadFeedbackClassName =
    downloadFeedback && downloadFeedback.validationIssueCount > 0
      ? "feedbackCard feedbackWarning"
      : "feedbackCard";
  const currentSessionDraft = session?.sourceDraft ?? null;

  return (
    <main className="appShell">
      <div className="appBackdrop" />
      <header className="appHeader">
        <div>
          <p className="eyebrow">PatentScribe AI Demo Slice</p>
          <h1>专利交底书优化工作台</h1>
          <p className="headerSummary">
            以左侧对话驱动右侧交底书预览更新。Tauri 桌面端检测到可用模型配置文件后会自动切到实时分析，否则回退为本地 mock 会话。
          </p>
        </div>
        <button
          type="button"
          className="ghostButton"
          onClick={() => void bootstrapSession(activePdfMode, sourceDraft)}
          disabled={isBusy}
        >
          重新载入演示
        </button>
      </header>

      <section className="workspaceGrid">
        <div className="panel panel-chat">
          <div className="panelHeader">
            <div>
              <p className="panelEyebrow">对话区</p>
              <h2>工程师与 AI 协作</h2>
            </div>
            <div className="panelStatus">
              <span className="metricCard">
                {session
                  ? `第 ${Math.min(
                      session.progress.currentQuestionIndex + 1,
                      session.progress.totalQuestions,
                    )} / ${session.progress.totalQuestions} 轮`
                  : "准备中"}
              </span>
            </div>
          </div>

          <div className="supportGrid">
            <SourceDraftPanel
              value={sourceDraft}
              currentSessionDraft={currentSessionDraft}
              onChange={setSourceDraft}
            />
            <ModelConfigPanel
              status={modelProviderStatus}
              isLoading={isLoadingConfigStatus}
              isRevealing={isRevealingConfigPath}
              onRefresh={() => void refreshModelProviderStatus()}
              onReveal={() => void handleRevealModelProviderConfig()}
            />
          </div>

          <div className="timelineWrap">
            {session ? (
              <ConversationTimeline messages={session.messages} />
            ) : (
              <div className="emptyState">正在准备演示会话...</div>
            )}
          </div>

          <Composer
            value={composerValue}
            onChange={setComposerValue}
            onSubmit={() => void handleSubmit()}
            onSkip={() => void handleSkip()}
            canSubmit={canSubmit}
            canSkip={canSkip}
            isBusy={isBusy}
            stage={session?.stage ?? "analyzing"}
          />

          {errorMessage ? <p className="feedbackCard feedbackError">{errorMessage}</p> : null}
        </div>

        <div className="panel panel-preview">
          <div className="panelHeader panelHeader-preview">
            <div>
              <p className="panelEyebrow">实时预览区</p>
              <h2>交底书当前版本</h2>
            </div>
            <div className="panelStatus stackStatus">
              <span className="metricCard">
                {session ? `版本 ${session.disclosure.version}` : "等待载入"}
              </span>
              <span className="metricCard">{activePdfMode === "direct_to_model" ? "PDF: 直传大模型" : "PDF: 本地预处理"}</span>
            </div>
          </div>

          <AnnotationLegend />

          <PreviewControls
            stage={session?.stage ?? "analyzing"}
            pdfMode={activePdfMode}
            zoom={zoom}
            agentView={agentView}
            showHighlights={showHighlights}
            annotationCount={annotationCount}
            validationIssueCount={validationIssueCount}
            includeAnswerHistory={includeAnswerHistory}
            canDownload={canDownload}
            canTerminate={canTerminate}
            isBusy={isBusy}
            onPdfModeChange={(mode) => void handlePdfModeChange(mode)}
            onZoomChange={(nextZoom) =>
              setZoom(Math.max(85, Math.min(nextZoom, 130)))
            }
            onAgentViewChange={setAgentView}
            onToggleHighlights={() => setShowHighlights((current) => !current)}
            onIncludeAnswerHistoryChange={setIncludeAnswerHistory}
            onDownload={() => void handleDownload()}
            onTerminate={() => void handleTerminate()}
          />

          <ValidationSummary issues={session?.validationIssues ?? []} />

          <div className="previewWrap">
            {session ? (
              <PreviewPanel
                disclosure={session.disclosure}
                agentView={agentView}
                showHighlights={showHighlights}
                zoom={zoom}
              />
            ) : (
              <div className="emptyState">等待加载交底书预览...</div>
            )}
          </div>

          {downloadFeedback ? (
            <p className={downloadFeedbackClassName}>
              {`${downloadFeedback.fileName}：${downloadFeedback.message}`}
            </p>
          ) : null}
        </div>
      </section>
    </main>
  );
}

export default App;
