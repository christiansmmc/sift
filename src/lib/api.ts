import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  Profile, Job, Application, PendingAction, DashboardCounts, Criteria,
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
  analyzeCv: (cvText: string) => invoke<Criteria>("analyze_cv", { cv_text: cvText }),
  saveLinkedinCredentials: (username: string, password: string) =>
    invoke<void>("save_linkedin_credentials", { username, password }),
  hasLinkedinCredentials: () => invoke<boolean>("has_linkedin_credentials"),
  getLinkedinUsername: () => invoke<string | null>("get_linkedin_username"),
};

export async function pickResumeFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: "Currículo", extensions: ["pdf", "docx"] }],
  });
  return typeof result === "string" ? result : null;
}
