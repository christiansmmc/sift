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

function parseAnswers(json: string): { question: string; answer: string }[] {
  try { return JSON.parse(json); } catch { return []; }
}

type VagasTab = "aguardando" | "aprovadas" | "encontradas";

export default function Jobs() {
  const [review, setReview] = useState<ReviewItem[]>([]);
  const [approved, setApproved] = useState<ReviewItem[]>([]);
  const [found, setFound] = useState<Job[]>([]);
  const [tab, setTab] = useState<VagasTab>("aguardando");

  async function refresh() {
    setReview(await api.listReviewQueue());
    setApproved(await api.listApproved());
    setFound(await api.listFoundJobs());
  }
  useEffect(() => { refresh(); }, []);

  async function approve(id: number) { await api.approveApplication(id); await refresh(); }
  async function reject(id: number) { await api.rejectApplication(id); await refresh(); }

  return (
    <section className="vagas-screen">
      <div className="vagas-header">
        <h1 className="vagas-title">Vagas</h1>
        <p className="vagas-subtitle">Revise e aprove candidaturas geradas pelo agente.</p>
      </div>

      <div className="tab-bar" style={{ marginBottom: 20 }}>
        <button
          className={`tab-btn${tab === "aguardando" ? " active" : ""}`}
          onClick={() => setTab("aguardando")}
        >
          Aguardando
          <span className="tab-count">{review.length}</span>
        </button>
        <button
          className={`tab-btn${tab === "aprovadas" ? " active" : ""}`}
          onClick={() => setTab("aprovadas")}
        >
          Aprovadas
          <span className="tab-count">{approved.length}</span>
        </button>
        <button
          className={`tab-btn${tab === "encontradas" ? " active" : ""}`}
          onClick={() => setTab("encontradas")}
        >
          Encontradas
          <span className="tab-count">{found.length}</span>
        </button>
      </div>

      {tab === "aguardando" && (
        <div className="vagas-tab-content">
          {review.length === 0 && (
            <p className="hint vagas-empty">Nada para revisar agora.</p>
          )}
          {review.map((r) => (
            <ReviewCard
              key={r.application_id}
              item={r}
              onApprove={() => approve(r.application_id)}
              onReject={() => reject(r.application_id)}
            />
          ))}
        </div>
      )}

      {tab === "aprovadas" && (
        <div className="vagas-tab-content">
          {approved.length === 0 && (
            <p className="hint vagas-empty">Nenhuma candidatura aprovada aguardando envio.</p>
          )}
          {approved.map((r) => (
            <ApprovedCard key={r.application_id} item={r} />
          ))}
        </div>
      )}

      {tab === "encontradas" && (
        <div className="vagas-tab-content">
          {found.length === 0 && (
            <p className="hint vagas-empty">Nenhuma vaga só-descoberta.</p>
          )}
          {found.length > 0 && (
            <div className="card vagas-found-card">
              {found.map((j, idx) => (
                <div
                  key={j.id}
                  className={`vagas-found-row${idx < found.length - 1 ? " vagas-found-row--bordered" : ""}`}
                >
                  <div className="vagas-found-main">
                    <a
                      href={j.url}
                      onClick={(e) => openExternal(e, j.url)}
                      className="vagas-found-title"
                    >
                      {j.title}
                    </a>
                    <span className="vagas-found-company">{j.company}</span>
                    <span className={`pill pill-${j.status}`}>{j.status}</span>
                  </div>
                  {j.match_summary && (
                    <p className="vagas-found-summary">{j.match_summary}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </section>
  );
}

function ApprovedCard({ item }: { item: ReviewItem }) {
  const answers = parseAnswers(item.answers_json);
  return (
    <div className="card vagas-card">
      <div className="vagas-card-header">
        <div className="vagas-card-meta">
          <strong className="vagas-card-title">{item.job_title}</strong>
          <span className="vagas-card-company">{item.company}</span>
          <a
            href={item.url}
            onClick={(e) => openExternal(e, item.url)}
            className="vagas-card-link"
          >
            ver vaga
          </a>
        </div>
        <span className="pill pill-approved">aprovada</span>
      </div>

      <div className="vagas-section-label">Carta de apresentação</div>
      <div className="cover-view">{item.cover_letter}</div>

      {answers.length > 0 && (
        <>
          <div className="vagas-section-label" style={{ marginTop: 14 }}>
            Respostas
          </div>
          <ul className="vagas-answers-list">
            {answers.map((a, i) => (
              <li key={i} className="vagas-answer-item">
                <span className="vagas-answer-q">{a.question}</span>
                <span className="vagas-answer-sep">—</span>
                <span className="vagas-answer-a">{a.answer}</span>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
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
  const [editing, setEditing] = useState(false);
  const [letter, setLetter] = useState(item.cover_letter);
  const [answers, setAnswers] = useState(initialAnswers);
  const [status, setStatus] = useState<string | null>(null);

  function setAnswer(i: number, v: string) {
    setAnswers((a) => a.map((x, idx) => (idx === i ? { ...x, answer: v } : x)));
  }
  async function persist() {
    await api.updateApplicationContent(item.application_id, letter, JSON.stringify(answers));
  }
  async function save() {
    setStatus(null);
    try { await persist(); setEditing(false); setStatus("Edições salvas."); }
    catch (e) { setStatus(`Erro: ${e}`); }
  }
  function cancel() {
    setLetter(item.cover_letter);
    setAnswers(initialAnswers);
    setEditing(false);
    setStatus(null);
  }
  async function approve() {
    try { await persist(); onApprove(); }
    catch (e) { setStatus(`Erro: ${e}`); }
  }

  return (
    <div className="card vagas-card">
      <div className="vagas-card-header">
        <div className="vagas-card-meta">
          <strong className="vagas-card-title">{item.job_title}</strong>
          <span className="vagas-card-company">{item.company}</span>
          <a
            href={item.url}
            onClick={(e) => openExternal(e, item.url)}
            className="vagas-card-link"
          >
            ver vaga
          </a>
        </div>
        <span className="pill pill-awaiting_approval">aguardando</span>
      </div>

      <div className="vagas-section-label" style={{ marginTop: 14 }}>
        Carta de apresentação
      </div>

      {editing ? (
        <>
          <textarea
            className="editing"
            rows={10}
            value={letter}
            onChange={(e) => setLetter(e.target.value)}
          />
          {answers.length > 0 && (
            <>
              <div className="vagas-section-label" style={{ marginTop: 14 }}>
                Respostas
              </div>
              {answers.map((a, i) => (
                <label className="field" key={i}>
                  {a.question}
                  <input value={a.answer} onChange={(e) => setAnswer(i, e.target.value)} />
                </label>
              ))}
            </>
          )}
        </>
      ) : (
        <>
          <div className="cover-view">{letter}</div>
          {answers.length > 0 && (
            <>
              <div className="vagas-section-label" style={{ marginTop: 14 }}>
                Respostas
              </div>
              <ul className="vagas-answers-list">
                {answers.map((a, i) => (
                  <li key={i} className="vagas-answer-item">
                    <span className="vagas-answer-q">{a.question}</span>
                    <span className="vagas-answer-sep">—</span>
                    <span className="vagas-answer-a">{a.answer}</span>
                  </li>
                ))}
              </ul>
            </>
          )}
        </>
      )}

      <div className="vagas-actions">
        <button className="btn btn-primary" onClick={approve}>Aprovar</button>
        <button className="btn btn-danger" onClick={onReject}>Rejeitar</button>
        {editing ? (
          <>
            <button className="btn" onClick={save}>Salvar edição</button>
            <button className="btn btn-ghost" onClick={cancel}>Cancelar</button>
          </>
        ) : (
          <button className="btn btn-ghost" onClick={() => setEditing(true)}>✏️ Editar</button>
        )}
        {status && <span className="hint">{status}</span>}
      </div>
    </div>
  );
}
