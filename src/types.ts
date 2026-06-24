export interface Criteria {
  role: string;
  seniority: string;
  work_model: string;
  locations: string[];
  salary_min: number | null;
  red_lines: string[];
}

export interface PersonalData {
  full_name: string;
  email: string;
  phone: string;
  location: string;
}

export interface CvAnalysis {
  personal: PersonalData;
  criteria: Criteria;
}

export interface Screening {
  english_level: string;
  salary_expectation: string;
  salary_currency: string;
  address: string;
  postal_code: string;
  work_authorization: string;
  availability: string;
}

export interface Profile {
  full_name: string;
  email: string;
  phone: string;
  location: string;
  cv_text: string;
  criteria_json: string;
  screening_json: string;
}

export interface Job {
  id: number;
  title: string;
  company: string;
  url: string;
  source: string;
  status: string;
  match_summary: string | null;
  discovered_at: string;
}

export interface Application {
  id: number;
  job_id: number;
  folder_path: string | null;
  cv_path: string | null;
  cover_letter_path: string | null;
  status: string;
  submitted_at: string | null;
}

export interface PendingAction {
  id: number;
  job_id: number | null;
  category: string;
  description: string;
  resolved: boolean;
  created_at: string;
  questions: string[];
}

export interface DashboardCounts {
  found: number;
  awaiting_approval: number;
  submitted: number;
  pending: number;
}
