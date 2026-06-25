import type { DashboardCounts } from "../types";

interface Props {
  counts: DashboardCounts | null;
  running: boolean;
  mode: "scan" | "revisar";
  batch: number;
  feed: string[];
  error: string | null;
  setMode: (m: "scan" | "revisar") => void;
  setBatch: (n: number) => void;
  onStart: () => void;
  onStop: () => void;
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="card" style={{ flex: 1, margin: 0, textAlign: "center" }}>
      <div style={{ fontSize: 28, fontWeight: 700 }}>{value}</div>
      <div className="hint">{label}</div>
    </div>
  );
}

export default function Dashboard({
  counts, running, mode, batch, feed, error, setMode, setBatch, onStart, onStop,
}: Props) {
  return (
    <section>
      <h1>Painel</h1>

      <div className="card">
        <div style={{ display: "flex", gap: 16, alignItems: "flex-end", flexWrap: "wrap" }}>
          <label className="field" style={{ marginBottom: 0 }}>
            Modo
            <select
              value={mode}
              onChange={(e) => setMode(e.target.value as "scan" | "revisar")}
              disabled={running}
            >
              <option value="revisar">Revisar (preparar p/ aprovar)</option>
              <option value="scan">Scan (só descobrir)</option>
            </select>
          </label>
          <label className="field" style={{ marginBottom: 0, width: 130 }}>
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
          {running ? (
            <button className="btn" onClick={onStop}>Parar</button>
          ) : (
            <button className="btn btn-primary" onClick={onStart}>Iniciar</button>
          )}
          <span className="hint" style={{ alignSelf: "center" }}>
            {running ? "🟢 Buscando…" : "⚪ Parado"}
          </span>
        </div>
        {error && <p className="hint" style={{ color: "var(--danger)" }}>{error}</p>}
      </div>

      {counts && (
        <div style={{ display: "flex", gap: 12, margin: "12px 0" }}>
          <Stat label="Encontradas" value={counts.found} />
          <Stat label="Aguardando aprovação" value={counts.awaiting_approval} />
          <Stat label="Enviadas" value={counts.submitted} />
          <Stat label="Pendências" value={counts.pending} />
        </div>
      )}

      {feed.length > 0 && (
        <div className="card">
          <h2>Atividade</h2>
          <div
            style={{
              maxHeight: 220,
              overflowY: "auto",
              display: "flex",
              flexDirection: "column",
              gap: 4,
              fontSize: 13,
            }}
          >
            {feed.map((line, i) => (
              <div key={i} style={{ color: "var(--text-muted)" }}>
                <span style={{ color: "var(--accent)" }}>›</span> {line}
              </div>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
