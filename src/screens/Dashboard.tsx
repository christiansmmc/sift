import { useEffect, useRef } from "react";
import type { DashboardCounts } from "../types";

interface Props {
  counts: DashboardCounts | null;
  running: boolean;
  runKind: "search" | "submit";
  mode: "scan" | "revisar";
  batch: number;
  feed: string[];
  error: string | null;
  setMode: (m: "scan" | "revisar") => void;
  setBatch: (n: number) => void;
  onStart: () => void;
  onStop: () => void;
  approvedCount: number;
  onSubmitApproved: () => void;
}

export default function Dashboard({
  counts, running, runKind, mode, batch, feed, error, setMode, setBatch, onStart, onStop,
  approvedCount, onSubmitApproved,
}: Props) {
  const statusLabel = running
    ? (runKind === "submit" ? "Enviando…" : "Buscando…")
    : "Parado";
  const statusActive = running;

  const feedRef = useRef<HTMLDivElement>(null);
  // Keep the activity feed scrolled to the newest entry as it grows.
  useEffect(() => {
    const el = feedRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [feed]);

  return (
    <section>
      {/* Page header */}
      <div className="painel-header">
        <h1>Painel</h1>
        <p className="painel-subtitle">Controle o agente e acompanhe suas candidaturas.</p>
      </div>

      {/* Controls card */}
      <div className="card painel-controls-card">
        <label className="field painel-mode-field">
          Modo
          <div className="painel-select-wrap">
            <select
              value={mode}
              onChange={(e) => setMode(e.target.value as "scan" | "revisar")}
              disabled={running}
            >
              <option value="revisar">Revisar (preparar p/ aprovar)</option>
              <option value="scan">Apenas buscar vagas</option>
            </select>
            <span className="painel-chevron">▾</span>
          </div>
        </label>

        <label className="field painel-batch-field">
          Vagas por busca
          <input
            type="number"
            min={1}
            max={50}
            value={batch}
            onChange={(e) => setBatch(Number(e.target.value))}
            disabled={running}
          />
        </label>

        <button
          className={`painel-run-btn${running ? " running" : ""}`}
          onClick={running ? onStop : onStart}
        >
          {running ? "Parar" : "Iniciar"}
        </button>

        {!running && approvedCount > 0 && (
          <button className="btn btn-primary" onClick={onSubmitApproved}>
            Enviar aprovadas ({approvedCount})
          </button>
        )}

        <div className="painel-status">
          <div className={`painel-status-dot${statusActive ? " painel-status-dot--active" : ""}`} />
          <span className="painel-status-label">{statusLabel}</span>
        </div>

        {error && <p className="painel-error">{error}</p>}
      </div>

      {/* Count cards */}
      {counts && (
        <div className="painel-counts">
          <div className="card painel-count-card">
            <div className="painel-count-card-label">
              <div className="painel-count-dot painel-count-dot--info" />
              <span>Encontradas</span>
            </div>
            <div className="painel-count-value">{counts.found}</div>
          </div>
          <div className="card painel-count-card">
            <div className="painel-count-card-label">
              <div className="painel-count-dot painel-count-dot--warn" />
              <span>Aguardando aprovação</span>
            </div>
            <div className="painel-count-value">{counts.awaiting_approval}</div>
          </div>
          <div className="card painel-count-card">
            <div className="painel-count-card-label">
              <div className="painel-count-dot painel-count-dot--ok" />
              <span>Enviadas</span>
            </div>
            <div className="painel-count-value">{counts.submitted}</div>
          </div>
          <div className="card painel-count-card">
            <div className="painel-count-card-label">
              <div className="painel-count-dot painel-count-dot--faint" />
              <span>Pendências</span>
            </div>
            <div className="painel-count-value">{counts.pending}</div>
          </div>
        </div>
      )}

      {/* Activity feed */}
      {feed.length > 0 && (
        <div className="painel-activity">
          <div className="painel-section-label">Atividade recente</div>
          <div className="card painel-feed-card" ref={feedRef}>
            {feed.map((line, i) => (
              <div key={i} className={`painel-feed-row${i < feed.length - 1 ? " painel-feed-row--bordered" : ""}`}>
                <div
                  className={`painel-feed-dot${
                    i === feed.length - 1
                      ? running
                        ? " painel-feed-dot--live"
                        : " painel-feed-dot--last"
                      : ""
                  }`}
                />
                <div className="painel-feed-line">{line}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
