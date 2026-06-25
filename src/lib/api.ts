import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  Profile, Job, Application, PendingAction, DashboardCounts, CvAnalysis, ReviewItem,
} from "../types";

export const api = {
  getOnboardingStatus: () => invoke<boolean>("get_onboarding_status"),
  getProfile: () => invoke<Profile>("get_profile"),
  saveProfile: (profile: Profile) => invoke<void>("save_profile", { profile }),
  listJobs: () => invoke<Job[]>("list_jobs"),
  listApplications: () => invoke<Application[]>("list_applications"),
  listPending: () => invoke<PendingAction[]>("list_pending"),
  resolvePending: (id: number) => invoke<void>("resolve_pending", { id }),
  listAnswers: () => invoke<{ question: string; answer: string }[]>("list_answers"),
  saveAnswer: (question: string, answer: string) =>
    invoke<void>("save_answer", { question, answer }),
  dashboardCounts: () => invoke<DashboardCounts>("dashboard_counts"),
  parseResume: (path: string) => invoke<string>("parse_resume", { path }),
  // Send both key cases: Tauri reads command args by exact name and ignores
  // extras, so this works whether it expects camelCase (cvText) or snake_case
  // (cv_text). Avoids a silent runtime arg-mismatch on this multi-word param.
  analyzeCv: (cvText: string) =>
    invoke<CvAnalysis>("analyze_cv", { cvText, cv_text: cvText }),
  startSearchBatch: (mode: string, batchSize: number) =>
    invoke<void>("start_search_batch", { mode, batchSize }),
  listReviewQueue: () => invoke<ReviewItem[]>("list_review_queue"),
  listFoundJobs: () => invoke<Job[]>("list_found_jobs"),
  approveApplication: (id: number) => invoke<void>("approve_application", { id }),
  rejectApplication: (id: number) => invoke<void>("reject_application", { id }),
  stopAgent: () => invoke<void>("stop_agent"),
  agentRunning: () => invoke<boolean>("agent_running"),
};

export async function pickResumeFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: "Currículo", extensions: ["pdf", "docx"] }],
  });
  return typeof result === "string" ? result : null;
}

export function onAgentEvent(cb: (payload: string) => void) {
  return listen<string>("agent://event", (e) => cb(e.payload));
}

export function onAgentStatus(cb: (text: string) => void) {
  return listen<string>("agent://status", (e) => cb(e.payload));
}
