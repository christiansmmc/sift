import { useEffect, useState } from "react";
import { api, onAgentEvent } from "../lib/api";
import type { DashboardCounts } from "../types";

export default function Dashboard() {
  const [counts, setCounts] = useState<DashboardCounts | null>(null);
  const [running, setRunning] = useState(false);
  const [batch, setBatch] = useState(10);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setCounts(await api.dashboardCounts());
    setRunning(await api.agentRunning());
  }

  useEffect(() => {
    refresh();
    const un = onAgentEvent(() => refresh());
    return () => { un.then((f) => f()); };
  }, []);

  async function start() {
    setError(null);
    try { await api.startSearchBatch(batch); setRunning(true); }
    catch (e) { setError(String(e)); }
  }
  async function stop() {
    await api.stopAgent();
    setRunning(false);
  }

  return (
    <section>
      <h1>Painel</h1>
      <div style={{ display: "flex", gap: 12, alignItems: "center", margin: "16px 0" }}>
        <label>Vagas por busca
          <input type="number" min={1} max={50} value={batch}
            onChange={(e) => setBatch(Number(e.target.value))}
            disabled={running} style={{ width: 64, marginLeft: 8 }} />
        </label>
        {running
          ? <button onClick={stop}>Parar</button>
          : <button onClick={start}>Iniciar</button>}
        <span>{running ? "🟢 Buscando…" : "⚪ Parado"}</span>
      </div>
      {error && <p style={{ color: "#c0392b" }}>{error}</p>}
      {counts && (
        <ul>
          <li>Vagas encontradas: {counts.found}</li>
          <li>Aguardando aprovação: {counts.awaiting_approval}</li>
          <li>Enviadas: {counts.submitted}</li>
          <li>Pendências: {counts.pending}</li>
        </ul>
      )}
    </section>
  );
}
