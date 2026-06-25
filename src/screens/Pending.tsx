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
    return (
      <section>
        <div className="pend-header">
          <h1 className="pend-title">Pendências</h1>
          <p className="pend-subtitle">Quando o agente precisar de você — uma pergunta sem resposta, login ou verificação — aparece aqui.</p>
        </div>
        <div className="card pend-empty-card">
          <div className="pend-empty-icon">
            <svg width="24" height="24" viewBox="0 0 24 24">
              <polyline points="5,12.5 10,17.5 19,7" fill="none" stroke="var(--ok)" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </div>
          <div className="pend-empty-title">Nada pendente</div>
          <div className="pend-empty-body">O agente está rodando sem precisar da sua intervenção.</div>
        </div>
      </section>
    );
  }

  return (
    <section>
      <div className="pend-header">
        <h1 className="pend-title">Pendências</h1>
        <p className="pend-subtitle">Quando o agente precisar de você — uma pergunta sem resposta, login ou verificação — aparece aqui.</p>
      </div>
      {status && <p className="hint">{status}</p>}
      {items.map((p) => (
        <div key={p.id} className="card">
          <div className="pend-card-pill">
            <span className={`pill pill-${p.category}`}>{labelFor(p.category)}</span>
          </div>
          <p className="pend-card-desc">{p.description}</p>
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
