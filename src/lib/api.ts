import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  Profile, Job, Application, PendingAction, DashboardCounts, CvAnalysis,
} from "../types";

export const api = {
  getOnboardingStatus: () => invoke<boolean>("get_onboarding_status"),
  getProfile: () => invoke<Profile>("get_profile"),
  saveProfile: (profile: Profile) => invoke<void>("save_profile", { profile }),
  listJobs: () => invoke<Job[]>("list_jobs"),
  listApplications: () => invoke<Application[]>("list_applications"),
  listPending: () => invoke<PendingAction[]>("list_pending"),
  resolvePending: (id: number) => invoke<void>("resolve_pending", { id }),
  dashboardCounts: () => invoke<DashboardCounts>("dashboard_counts"),
  parseResume: (path: string) => invoke<string>("parse_resume", { path }),
  // Send both key cases: Tauri reads command args by exact name and ignores
  // extras, so this works whether it expects camelCase (cvText) or snake_case
  // (cv_text). Avoids a silent runtime arg-mismatch on this multi-word param.
  analyzeCv: (cvText: string) =>
    invoke<CvAnalysis>("analyze_cv", { cvText, cv_text: cvText }),
};

export async function pickResumeFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: "Currículo", extensions: ["pdf", "docx"] }],
  });
  return typeof result === "string" ? result : null;
}
