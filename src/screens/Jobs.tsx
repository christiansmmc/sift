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
  const [found, setFound] = useState<Job[]>([]);

  async function refresh() {
    setReview(await api.listReviewQueue());
    setFound(await api.listFoundJobs());
  }
  useEffect(() => { refresh(); }, []);

  async function approve(id: number) { await api.approveApplication(id); await refresh(); }
  async function reject(id: number) { await api.rejectApplication(id); await refresh(); }

  function answers(json: string): { question: string; answer: string }[] {
    try { return JSON.parse(json); } catch { return []; }
  }

  return (
    <section>
      <h1>Vagas</h1>

      <h2>Aguardando aprovação ({review.length})</h2>
      {review.length === 0 && <p className="hint">Nada para revisar agora.</p>}
      {review.map((r) => (
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
          {answers(r.answers_json).length > 0 && (
            <details>
              <summary>Respostas ({answers(r.answers_json).length})</summary>
              <ul>{answers(r.answers_json).map((a, i) => <li key={i}><b>{a.question}</b> — {a.answer}</li>)}</ul>
            </details>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
            <button className="btn btn-primary" onClick={() => approve(r.application_id)}>Aprovar</button>
            <button className="btn btn-ghost" onClick={() => reject(r.application_id)}>Rejeitar</button>
          </div>
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
