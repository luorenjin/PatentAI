export type MessageRole = "system" | "assistant" | "engineer";

export type ModelProviderStatusKind =
  | "ready"
  | "needs_configuration"
  | "invalid_config"
  | "unsupported_provider"
  | "browser_mock";

export interface ModelProviderStatus {
  status: ModelProviderStatusKind;
  message: string;
  configPath: string;
  createdTemplate: boolean;
  provider?: string | null;
  apiUrl?: string | null;
  modelName?: string | null;
  hasApiKey: boolean;
  apiKeyPreview?: string | null;
}

export type MessageIntent =
  | "status"
  | "diagnosis"
  | "follow_up"
  | "answer"
  | "advisory"
  | "error";

export type SessionStage =
  | "idle"
  | "analyzing"
  | "awaiting_answer"
  | "updating_preview"
  | "advisory_complete"
  | "terminated";

export type PdfProcessingMode = "direct_to_model" | "local_preprocess";

export type AnnotationKind = "addition" | "rewrite";

export interface ChatMessage {
  id: string;
  role: MessageRole;
  intent: MessageIntent;
  content: string;
  timestamp: string;
}

export interface DisclosureSection {
  id: string;
  title: string;
  content: string;
}

export interface Annotation {
  id: string;
  kind: AnnotationKind;
  sectionId: string;
  excerpt: string;
  note: string;
}

export interface DisclosureDocument {
  version: number;
  sections: DisclosureSection[];
  annotations: Annotation[];
}

export interface DemoSettings {
  pdfMode: PdfProcessingMode;
}

export interface SessionActions {
  canSubmit: boolean;
  canSkip: boolean;
  canDownload: boolean;
  canTerminate: boolean;
}

export type ValidationLevel = "error" | "warning";

export interface ValidationIssue {
  code: string;
  message: string;
  sectionId?: string | null;
  level: ValidationLevel;
}

export interface RetryMetadata {
  attemptCount: number;
  maxAttempts: number;
}

export interface SessionProgress {
  currentQuestionIndex: number;
  totalQuestions: number;
}

export interface SessionSnapshot {
  sessionId: string;
  stage: SessionStage;
  sourceDraft: string;
  messages: ChatMessage[];
  disclosure: DisclosureDocument;
  settings: DemoSettings;
  actions: SessionActions;
  progress: SessionProgress;
  validationIssues: ValidationIssue[];
  retryMetadata: RetryMetadata;
}

export interface StartDemoSessionRequest {
  settings: DemoSettings;
  sourceDraft: string;
}

export interface AnswerRequest {
  session: SessionSnapshot;
  answer: string;
}

export interface SessionRequest {
  session: SessionSnapshot;
}

export interface UpdateSettingsRequest {
  session: SessionSnapshot;
  settings: DemoSettings;
}

export interface DownloadRequest {
  session: SessionSnapshot;
  includeAnswerHistory: boolean;
}

export type DownloadStatus = "ready" | "placeholder";

export interface DownloadResponse {
  status: DownloadStatus;
  fileName: string;
  message: string;
  includedAnswerHistory: boolean;
  validationIssueCount: number;
}