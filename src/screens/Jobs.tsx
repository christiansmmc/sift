import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { Job, ReviewItem } from "../types";

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
        <div key={r.application_id} style={{ border: "1px solid #ddd", borderRadius: 8, padding: 12, margin: "12px 0" }}>
          <strong>{r.job_title}</strong> — {r.company}{" "}
          <a href={r.url} target="_blank" rel="noreferrer">ver vaga</a>
          <details style={{ margin: "8px 0" }}>
            <summary>Carta de apresentação</summary>
            <pre style={{ whiteSpace: "pre-wrap", fontFamily: "inherit" }}>{r.cover_letter}</pre>
          </details>
          {answers(r.answers_json).length > 0 && (
            <details>
              <summary>Respostas ({answers(r.answers_json).length})</summary>
              <ul>{answers(r.answers_json).map((a, i) => <li key={i}><b>{a.question}</b> — {a.answer}</li>)}</ul>
            </details>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
            <button onClick={() => approve(r.application_id)}>Aprovar</button>
            <button onClick={() => reject(r.application_id)}>Rejeitar</button>
          </div>
        </div>
      ))}

      <h2 style={{ marginTop: 24 }}>Encontradas — Scan ({found.length})</h2>
      {found.length === 0 && <p className="hint">Nenhuma vaga só-descoberta.</p>}
      <ul>
        {found.map((j) => (
          <li key={j.id}>
            <a href={j.url} target="_blank" rel="noreferrer">{j.title}</a> — {j.company}
            {j.match_summary ? ` · ${j.match_summary}` : ""}
          </li>
        ))}
      </ul>
    </section>
  );
}
