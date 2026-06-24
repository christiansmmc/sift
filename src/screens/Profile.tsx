import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile as ProfileT } from "../types";
import "../profile.css";

const EMPTY_CRITERIA: Criteria = {
  role: "", seniority: "", work_model: "", locations: [], salary_min: null, red_lines: [],
};

function parseCriteria(json: string): Criteria {
  try { return { ...EMPTY_CRITERIA, ...JSON.parse(json) }; } catch { return EMPTY_CRITERIA; }
}

export default function Profile() {
  const [profile, setProfile] = useState<ProfileT | null>(null);
  const [criteria, setCriteria] = useState<Criteria>(EMPTY_CRITERIA);
  const [linkedinUser, setLinkedinUser] = useState<string>("");
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.getProfile().then((p) => {
      setProfile(p);
      setCriteria(parseCriteria(p.criteria_json));
    });
    api.getLinkedinUsername().then((u) => setLinkedinUser(u ?? ""));
  }, []);

  if (!profile) return <section className="prof"><h1>Perfil</h1><p>Carregando…</p></section>;

  const setField = (k: keyof ProfileT, v: string) => setProfile({ ...profile, [k]: v });
  const setCrit = <K extends keyof Criteria,>(k: K, v: Criteria[K]) => setCriteria({ ...criteria, [k]: v });

  async function analyze() {
    if (!profile!.cv_text.trim()) return;
    setBusy(true); setStatus(null);
    try {
      const a = await api.analyzeCv(profile!.cv_text);
      const gotCriteria = a.criteria.role || a.criteria.seniority || a.criteria.work_model;
      if (gotCriteria) {
        setCriteria({ ...criteria, ...a.criteria });
        // fill only personal fields that are currently empty, never overwrite
        setProfile((prev) => prev && {
          ...prev,
          full_name: prev.full_name || a.personal.full_name,
          email: prev.email || a.personal.email,
          phone: prev.phone || a.personal.phone,
          location: prev.location || a.personal.location,
        });
        setStatus("Critérios atualizados pela análise. Revise e salve.");
      } else {
        setStatus("Não consegui inferir critérios — ajuste manualmente.");
      }
    } catch (e) {
      setStatus(`Análise indisponível (${e}).`);
    } finally { setBusy(false); }
  }

  async function save() {
    setBusy(true); setStatus(null);
    try {
      await api.saveProfile({ ...profile!, criteria_json: JSON.stringify(criteria) });
      setStatus("Perfil salvo.");
    } catch (e) {
      setStatus(`Erro ao salvar: ${e}`);
    } finally { setBusy(false); }
  }

  return (
    <section className="prof">
      <h1>Perfil</h1>

      <h2>Seus dados</h2>
      <label>Nome completo<input value={profile.full_name} onChange={(e) => setField("full_name", e.target.value)} /></label>
      <label>E-mail<input value={profile.email} onChange={(e) => setField("email", e.target.value)} /></label>
      <label>Telefone<input value={profile.phone} onChange={(e) => setField("phone", e.target.value)} /></label>
      <label>Localização<input value={profile.location} onChange={(e) => setField("location", e.target.value)} /></label>
      <label>LinkedIn<input value={linkedinUser} disabled title="Editável na reconfiguração de credenciais" /></label>

      <h2>Currículo</h2>
      <label><textarea rows={10} value={profile.cv_text} onChange={(e) => setField("cv_text", e.target.value)} /></label>
      <button onClick={analyze} disabled={busy || !profile.cv_text.trim()}>
        {busy ? "Analisando…" : "Analisar com Claude"}
      </button>

      <h2>O que você busca</h2>
      <label>Cargo<input value={criteria.role} onChange={(e) => setCrit("role", e.target.value)} /></label>
      <label>Senioridade<input value={criteria.seniority} onChange={(e) => setCrit("seniority", e.target.value)} placeholder="junior / mid / senior / lead" /></label>
      <label>Modelo de trabalho
        <select value={criteria.work_model} onChange={(e) => setCrit("work_model", e.target.value)}>
          <option value="">Indiferente</option>
          <option value="remote">Remoto</option>
          <option value="hybrid">Híbrido</option>
          <option value="onsite">Presencial</option>
        </select>
      </label>
      <label>Localizações (vírgula)
        <input value={criteria.locations.join(", ")}
          onChange={(e) => setCrit("locations", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <label>Salário mínimo (R$)
        <input type="number" value={criteria.salary_min ?? ""}
          onChange={(e) => setCrit("salary_min", e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>Red-lines (vírgula)
        <input value={criteria.red_lines.join(", ")}
          onChange={(e) => setCrit("red_lines", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>

      <div className="prof-actions">
        <button onClick={save} disabled={busy}>Salvar</button>
        {status && <span className="prof-status">{status}</span>}
      </div>
    </section>
  );
}
