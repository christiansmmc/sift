import { useState } from "react";
import { api, pickResumeFile } from "../../lib/api";
import type { Criteria } from "../../types";

type Personal = { full_name: string; email: string; phone: string; location: string };

// Merge incoming over base, skipping empty string / null / empty array values.
function mergeNonEmpty<T>(base: T, incoming: Partial<T>): T {
  const out = { ...base } as Record<string, unknown>;
  (Object.keys(incoming as object) as (keyof T)[]).forEach((k) => {
    const v = incoming[k] as unknown;
    const empty = v === "" || v === null || (Array.isArray(v) && v.length === 0);
    if (!empty && v !== undefined) out[k as string] = v;
  });
  return out as T;
}

export default function StepCv({
  cvText, setCvText, personal, setPersonal, criteria, setCriteria,
}: {
  cvText: string; setCvText: (t: string) => void;
  personal: Personal; setPersonal: (p: Personal) => void;
  criteria: Criteria; setCriteria: (c: Criteria) => void;
}) {
  const [busy, setBusy] = useState<"" | "parsing" | "analyzing">("");
  const [note, setNote] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [analyzed, setAnalyzed] = useState(false);

  async function upload() {
    setNote(null);
    const path = await pickResumeFile();
    if (!path) return;
    setBusy("parsing");
    setFileName(null);
    setAnalyzed(false);
    try {
      const text = await api.parseResume(path);
      setCvText(text);
      setFileName(path.split(/[/\\]/).pop() ?? path);
    } catch (e) {
      setNote(`Não consegui ler o arquivo: ${e}`);
    } finally {
      setBusy("");
    }
  }

  async function analyze() {
    if (!cvText.trim()) return;
    setBusy("analyzing");
    setNote(null);
    setAnalyzed(false);
    try {
      const a = await api.analyzeCv(cvText);
      const gotPersonal = a.personal.full_name || a.personal.email || a.personal.location;
      const gotCriteria = a.criteria.role || a.criteria.seniority || a.criteria.work_model;
      if (gotPersonal) setPersonal(mergeNonEmpty(personal, a.personal));
      if (gotCriteria) setCriteria(mergeNonEmpty(criteria, a.criteria));
      if (gotPersonal || gotCriteria) {
        setAnalyzed(true);
      } else {
        setNote("Não consegui extrair informações automaticamente — você pode preencher manualmente.");
      }
    } catch (e) {
      setNote(`Análise indisponível (${e}). Preencha manualmente.`);
    } finally {
      setBusy("");
    }
  }

  return (
    <section className="step">
      <div className="onb-eyebrow">Currículo</div>
      <p className="hint">Envie ou cole seu currículo. A análise pré-preenche seus dados e os critérios de busca — você revisa nos próximos passos.</p>
      <div className="row">
        <button className="btn" onClick={upload} disabled={busy !== ""}>
          <svg width="14" height="14" viewBox="0 0 14 14">
            <path d="M7 9.5V2.5M4.2 5l2.8-2.8L9.8 5M2.5 9.5v1.5a1 1 0 0 0 1 1h7a1 1 0 0 0 1-1V9.5" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          {busy === "parsing" ? "Lendo…" : "Enviar PDF/DOCX"}
        </button>
        <button className="btn" onClick={analyze} disabled={!cvText.trim() || busy !== ""}>
          {busy === "analyzing" && (
            <svg width="14" height="14" viewBox="0 0 14 14" className="onb-spin">
              <circle cx="7" cy="7" r="5.2" fill="none" stroke="currentColor" strokeWidth="1.4" strokeDasharray="22 12" strokeLinecap="round" />
            </svg>
          )}
          {busy === "analyzing" ? "Analisando…" : "Analisar com Claude"}
        </button>
        {fileName && (
          <span className="onb-filename">
            <svg width="12" height="12" viewBox="0 0 13 13">
              <polyline points="2.5,7 5,9.5 10.5,3.5" fill="none" stroke="var(--ok)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            {fileName}
          </span>
        )}
      </div>
      {analyzed && (
        <div className="onb-analyzed">
          <svg width="14" height="14" viewBox="0 0 13 13">
            <polyline points="2.5,7 5,9.5 10.5,3.5" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          Análise concluída — seus dados e critérios foram pré-preenchidos.
        </div>
      )}
      <label className="field">Texto do currículo
        <textarea rows={12} value={cvText} onChange={(e) => setCvText(e.target.value)}
          placeholder="Cole o texto do seu currículo aqui, ou envie um PDF/DOCX acima." />
      </label>
      {note && <p className="hint">{note}</p>}
    </section>
  );
}
