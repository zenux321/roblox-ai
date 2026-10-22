/** Puntea către shellul Tauri. În browser (preview, teste) totul este inert. */
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export interface ExampleConfigVerdict {
  name: string;
  contract_valid: boolean;
  execution_ready: boolean;
  execution_issue_codes: string[];
}

export interface RuntimeInfo {
  app_version: string;
  core_version: string;
  demo_mode: boolean;
  example_configs: ExampleConfigVerdict[];
}

export function inDesktopShell(): boolean {
  return isTauri();
}

export async function fetchRuntimeInfo(): Promise<RuntimeInfo | null> {
  if (!isTauri()) {
    return null;
  }
  return invoke<RuntimeInfo>('runtime_info');
}

export const windowControls = {
  minimize: () => getCurrentWindow().minimize(),
  toggleMaximize: () => getCurrentWindow().toggleMaximize(),
  close: () => getCurrentWindow().close(),
};
