import { useEffect, useState } from "react";
import { api, onAgentEvent, onAgentStatus } from "../lib/api";
import type { DashboardCounts } from "../types";

export default function Dashboard() {
  const [counts, setCounts] = useState<DashboardCounts | null>(null);
  const [running, setRunning] = useState(false);
  const [batch, setBatch] = useState(10);
  const [mode, setMode] = useState<"scan" | "revisar">("revisar");
  const [error, setError] = useState<string | null>(null);
  const [feed, setFeed] = useState<string[]>([]);

  async function refresh() {
    setCounts(await api.dashboardCounts());
    setRunning(await api.agentRunning());
  }

  useEffect(() => {
    refresh();
    const un = onAgentEvent(() => refresh());
    const unStatus = onAgentStatus((t) => setFeed((f) => [...f, t].slice(-50)));
    return () => {
      un.then((f) => f());
      unStatus.then((f) => f());
    };
  }, []);

  async function start() {
    setError(null);
    setFeed([]);
    try { await api.startSearchBatch(mode, batch); setRunning(true); }
    catch (e) { setError(String(e)); }
  }
  async function stop() {
    await api.stopAgent();
    setRunning(false);
  }

  return (
    <section>
      <h1>Painel</h1>

      <div className="card">
        <label className="field">
          Modo
          <select value={mode} onChange={(e) => setMode(e.target.value as "scan" | "revisar")} disabled={running}>
            <option value="revisar">Revisar (preparar p/ aprovar)</option>
            <option value="scan">Scan (só descobrir)</option>
          </select>
        </label>
        <label className="field">
          Vagas por busca
          <input type="number" min={1} max={50} value={batch}
            onChange={(e) => setBatch(Number(e.target.value))}
            disabled={running} style={{ width: 80 }} />
        </label>
        <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 8 }}>
          {running
            ? <button className="btn" onClick={stop}>Parar</button>
            : <button className="btn btn-primary" onClick={start}>Iniciar</button>}
          <span className="hint">{running ? "🟢 Buscando…" : "⚪ Parado"}</span>
        </div>
        {error && <p style={{ color: "var(--danger)", marginTop: 8 }}>{error}</p>}
      </div>

      {counts && (
        <div style={{ display: "flex", gap: 12, flexWrap: "wrap", marginTop: 8 }}>
          <div className="card" style={{ flex: "1 1 120px", textAlign: "center" }}>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{counts.found}</div>
            <div className="hint">Encontradas</div>
          </div>
          <div className="card" style={{ flex: "1 1 120px", textAlign: "center" }}>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{counts.awaiting_approval}</div>
            <div className="hint">Aguardando aprovação</div>
          </div>
          <div className="card" style={{ flex: "1 1 120px", textAlign: "center" }}>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{counts.submitted}</div>
            <div className="hint">Enviadas</div>
          </div>
          <div className="card" style={{ flex: "1 1 120px", textAlign: "center" }}>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{counts.pending}</div>
            <div className="hint">Pendências</div>
          </div>
        </div>
      )}

      {feed.length > 0 && (
        <div className="card" style={{ marginTop: 12 }}>
          <div style={{ fontWeight: 600, marginBottom: 6 }}>Atividade</div>
          <div style={{ maxHeight: 200, overflowY: "auto", display: "flex", flexDirection: "column", gap: 2 }}>
            {feed.map((line, i) => (
              <div key={i} style={{ fontSize: 13 }}>
                <span style={{ color: "var(--accent)" }}>›</span>{" "}
                <span className="hint">{line}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
