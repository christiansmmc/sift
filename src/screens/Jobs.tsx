import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../lib/api";
import type { Job, ReviewItem } from "../types";

// Open external links in the system browser (a plain <a target=_blank> does
// nothing inside the Tauri webview).
function openExternal(e: React.MouseEvent, url: string) {
  e.preventDefault();
  openUrl(url);
}

export default function Jobs() {
  const [review, setReview] = useState<ReviewItem[]>([]);
  const [approved, setApproved] = useState<ReviewItem[]>([]);
  const [found, setFound] = useState<Job[]>([]);

  async function refresh() {
    setReview(await api.listReviewQueue());
    setApproved(await api.listApproved());
    setFound(await api.listFoundJobs());
  }
  useEffect(() => { refresh(); }, []);

  async function approve(id: number) { await api.approveApplication(id); await refresh(); }
  async function reject(id: number) { await api.rejectApplication(id); await refresh(); }

  return (
    <section>
      <h1>Vagas</h1>

      <h2>Aguardando aprovação ({review.length})</h2>
      {review.length === 0 && <p className="hint">Nada para revisar agora.</p>}
      {review.map((r) => (
        <ReviewCard
          key={r.application_id}
          item={r}
          onApprove={() => approve(r.application_id)}
          onReject={() => reject(r.application_id)}
        />
      ))}

      <h2>Aprovadas (aguardando envio) ({approved.length})</h2>
      {approved.length === 0 && <p className="hint">Nenhuma candidatura aprovada aguardando envio.</p>}
      {approved.map((r) => (
        <div key={r.application_id} className="card">
          <div>
            <strong>{r.job_title}</strong>
            {" "}<span style={{ color: "var(--text-muted)" }}>{r.company}</span>
            {" "}<a href={r.url} onClick={(e) => openExternal(e, r.url)}>ver vaga</a>
          </div>
          <details style={{ margin: "8px 0" }}>
            <summary>Carta de apresentação</summary>
            <pre style={{ whiteSpace: "pre-wrap", fontFamily: "inherit", background: "var(--surface-2)", padding: "10px", borderRadius: "var(--radius)", marginTop: 8 }}>{r.cover_letter}</pre>
          </details>
        </div>
      ))}

      <h2>Encontradas — Scan ({found.length})</h2>
      {found.length === 0 && <p className="hint">Nenhuma vaga só-descoberta.</p>}
      {found.map((j) => (
        <div key={j.id} className="card" style={{ padding: "10px 16px" }}>
          <a href={j.url} onClick={(e) => openExternal(e, j.url)}><strong>{j.title}</strong></a>
          {" "}<span style={{ color: "var(--text-muted)" }}>{j.company}</span>
          {j.match_summary ? <span className="hint" style={{ display: "block", marginTop: 2 }}>{j.match_summary}</span> : null}
        </div>
      ))}
    </section>
  );
}

function ReviewCard({
  item, onApprove, onReject,
}: {
  item: ReviewItem;
  onApprove: () => void;
  onReject: () => void;
}) {
  const initialAnswers = (() => {
    try { return JSON.parse(item.answers_json) as { question: string; answer: string }[]; }
    catch { return []; }
  })();
  const [letter, setLetter] = useState(item.cover_letter);
  const [answers, setAnswers] = useState(initialAnswers);
  const [status, setStatus] = useState<string | null>(null);

  function setAnswer(i: number, v: string) {
    setAnswers((a) => a.map((x, idx) => (idx === i ? { ...x, answer: v } : x)));
  }
  async function save() {
    setStatus(null);
    try {
      await api.updateApplicationContent(item.application_id, letter, JSON.stringify(answers));
      setStatus("Edições salvas.");
    } catch (e) { setStatus(`Erro: ${e}`); }
  }
  async function approve() {
    try {
      await api.updateApplicationContent(item.application_id, letter, JSON.stringify(answers));
      onApprove();
    } catch (e) { setStatus(`Erro: ${e}`); }
  }

  return (
    <div className="card">
      <strong>{item.job_title}</strong> — {item.company}{" "}
      <a href={item.url} onClick={(e) => openExternal(e, item.url)}>ver vaga</a>
      <label className="field" style={{ marginTop: 12 }}>
        Carta de apresentação
        <textarea rows={10} value={letter} onChange={(e) => setLetter(e.target.value)} />
      </label>
      {answers.length > 0 && (
        <div>
          <div className="hint" style={{ marginBottom: 8 }}>Respostas</div>
          {answers.map((a, i) => (
            <label className="field" key={i}>
              {a.question}
              <input value={a.answer} onChange={(e) => setAnswer(i, e.target.value)} />
            </label>
          ))}
        </div>
      )}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn btn-primary" onClick={approve}>Aprovar</button>
        <button className="btn btn-ghost" onClick={onReject}>Rejeitar</button>
        <button className="btn" onClick={save}>Salvar edição</button>
        {status && <span className="hint">{status}</span>}
      </div>
    </div>
  );
}
