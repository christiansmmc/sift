import { useState } from "react";
import { api, pickResumeFile } from "../../lib/api";
import type { Criteria } from "../../types";

export default function StepCv({
  cvText, setCvText, criteria, setCriteria,
}: {
  cvText: string; setCvText: (t: string) => void;
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
      const c = await api.analyzeCv(cvText);
      if (c.role || c.seniority || c.work_model) {
        setCriteria({ ...criteria, ...c });
        setNote("Critérios pré-preenchidos a partir do currículo. Revise no próximo passo.");
      } else {
        setNote("Não consegui inferir critérios automaticamente — você pode preencher manualmente.");
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
