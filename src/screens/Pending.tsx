import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { PendingAction } from "../types";

export default function Pending() {
  const [items, setItems] = useState<PendingAction[]>([]);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<string | null>(null);

  async function refresh() {
    setItems(await api.listPending());
  }
  useEffect(() => { refresh(); }, []);

  function setDraft(key: string, v: string) {
    setDrafts((d) => ({ ...d, [key]: v }));
  }

  async function saveAnswers(p: PendingAction) {
    setStatus(null);
    try {
      for (const q of p.questions) {
        const key = `${p.id}:${q}`;
        const a = drafts[key]?.trim();
        if (a) await api.saveAnswer(q, a);
      }
      await api.resolvePending(p.id);
      setStatus("Respostas salvas — o agente vai usá-las na próxima busca.");
      await refresh();
    } catch (e) {
      setStatus(`Erro: ${e}`);
    }
  }

  async function dismiss(p: PendingAction) {
    await api.resolvePending(p.id);
    await refresh();
  }

  if (items.length === 0) {
    return <section><h1>Pendências</h1><p>Nada pendente. 🎉</p></section>;
  }

  return (
    <section>
      <h1>Pendências</h1>
      {status && <p className="hint">{status}</p>}
      {items.map((p) => (
        <div key={p.id} className="card">
          <div style={{ marginBottom: 8 }}>
            <span className={`pill pill-${p.category}`}>{labelFor(p.category)}</span>
          </div>
          <p style={{ color: "var(--text-muted)", margin: "4px 0 10px" }}>{p.description}</p>
          {p.questions.length > 0 ? (
            <>
              {p.questions.map((q) => {
                const key = `${p.id}:${q}`;
                return (
                  <label key={key} className="field">
                    {q}
                    <input value={drafts[key] ?? ""} onChange={(e) => setDraft(key, e.target.value)} />
                  </label>
                );
              })}
              <button className="btn btn-primary" onClick={() => saveAnswers(p)}>Salvar respostas e resolver</button>
            </>
          ) : (
            <button className="btn btn-ghost" onClick={() => dismiss(p)}>Resolver</button>
          )}
        </div>
      ))}
    </section>
  );
}

function labelFor(category: string): string {
  switch (category) {
    case "missing_answer": return "Falta resposta";
    case "login_required": return "Login necessário";
    case "external_application": return "Candidatura externa";
    case "captcha": return "Captcha";
    default: return category;
  }
}
