import type { InvokeArgs } from "@tauri-apps/api/core";
import type {
  AnswerRequest,
  DemoSettings,
  DownloadRequest,
  DownloadResponse,
  ModelProviderStatus,
  SessionRequest,
  SessionSnapshot,
  StartDemoSessionRequest,
  UpdateSettingsRequest,
} from "../types/patentSession";
import {
  downloadBrowserDemoSnapshot,
  skipBrowserDemoQuestion,
  startBrowserDemoSession,
  submitBrowserDemoAnswer,
  terminateBrowserDemoSession,
  updateBrowserDemoSettings,
} from "./browserDemoSession";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invokeCommand<T>(command: string, payload: object): Promise<T> {
  if (!isTauriRuntime()) {
    throw new Error("Tauri runtime unavailable");
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, payload as InvokeArgs);
}

export function startDemoSession(
  request: StartDemoSessionRequest,
): Promise<SessionSnapshot> {
  if (!isTauriRuntime()) {
    return startBrowserDemoSession(request);
  }

  return invokeCommand<SessionSnapshot>("start_demo_session", { request });
}

export function getModelProviderStatus(): Promise<ModelProviderStatus> {
  if (!isTauriRuntime()) {
    return Promise.resolve({
      status: "browser_mock",
      message:
        "当前页面运行在浏览器 mock 预览模式，模型配置文件只会在 Tauri 桌面端读取和校验。",
      configPath: "",
      createdTemplate: false,
      provider: null,
      apiUrl: null,
      modelName: null,
      hasApiKey: false,
      apiKeyPreview: null,
    });
  }

  return invokeCommand<ModelProviderStatus>("get_model_provider_status", {});
}

export async function revealModelProviderConfig(configPath: string): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("当前不是 Tauri 桌面端，无法打开本地配置文件位置。");
  }

  const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
  await revealItemInDir(configPath);
}

export function submitDemoAnswer(
  request: AnswerRequest,
): Promise<SessionSnapshot> {
  if (!isTauriRuntime()) {
    return submitBrowserDemoAnswer(request.session, request.answer);
  }

  return invokeCommand<SessionSnapshot>("submit_demo_answer", { request });
}

export function skipDemoQuestion(
  request: SessionRequest,
): Promise<SessionSnapshot> {
  if (!isTauriRuntime()) {
    return skipBrowserDemoQuestion(request.session);
  }

  return invokeCommand<SessionSnapshot>("skip_demo_question", { request });
}

export function updateDemoSettings(
  request: UpdateSettingsRequest,
): Promise<SessionSnapshot> {
  if (!isTauriRuntime()) {
    return updateBrowserDemoSettings(request);
  }

  return invokeCommand<SessionSnapshot>("update_demo_settings", { request });
}

export function terminateDemoSession(
  request: SessionRequest,
): Promise<SessionSnapshot> {
  if (!isTauriRuntime()) {
    return terminateBrowserDemoSession(request);
  }

  return invokeCommand<SessionSnapshot>("terminate_demo_session", { request });
}

export function downloadDemoSnapshot(
  request: DownloadRequest,
): Promise<DownloadResponse> {
  if (!isTauriRuntime()) {
    return downloadBrowserDemoSnapshot(request);
  }

  return invokeCommand<DownloadResponse>("download_demo_snapshot", { request });
}

export function buildDemoSettings(pdfMode: DemoSettings["pdfMode"]): DemoSettings {
  return { pdfMode };
}