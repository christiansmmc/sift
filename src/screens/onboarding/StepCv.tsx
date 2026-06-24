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

  async function upload() {
    setNote(null);
    const path = await pickResumeFile();
    if (!path) return;
    setBusy("parsing");
    try {
      const text = await api.parseResume(path);
      setCvText(text);
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
    try {
      const a = await api.analyzeCv(cvText);
      const gotPersonal = a.personal.full_name || a.personal.email || a.personal.location;
      const gotCriteria = a.criteria.role || a.criteria.seniority || a.criteria.work_model;
      if (gotPersonal) setPersonal(mergeNonEmpty(personal, a.personal));
      if (gotCriteria) setCriteria(mergeNonEmpty(criteria, a.criteria));
      if (gotPersonal || gotCriteria) {
        setNote("Dados e critérios pré-preenchidos a partir do currículo. Revise nos próximos passos.");
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
      <h2>Currículo</h2>
      <p className="hint">Envie ou cole seu currículo. A análise pré-preenche seus dados e os critérios de busca — você revisa nos próximos passos.</p>
      <div className="row">
        <button onClick={upload} disabled={busy !== ""}>
          {busy === "parsing" ? "Lendo…" : "Enviar PDF/DOCX"}
        </button>
        <button onClick={analyze} disabled={!cvText.trim() || busy !== ""}>
          {busy === "analyzing" ? "Analisando…" : "Analisar com Claude"}
        </button>
      </div>
      <label>Texto do currículo
        <textarea rows={12} value={cvText} onChange={(e) => setCvText(e.target.value)}
          placeholder="Cole o texto do seu currículo aqui, ou envie um PDF/DOCX acima." />
      </label>
      {note && <p className="hint">{note}</p>}
    </section>
  );
}
