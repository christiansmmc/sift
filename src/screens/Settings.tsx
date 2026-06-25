import React, { useEffect, useState } from "react";
import { api } from "../lib/api";

type Style = "short" | "balanced" | "detailed" | "custom";

const LABELS: Record<Style, string> = {
  short: "Curta e simples",
  balanced: "Equilibrada",
  detailed: "Detalhada (formal)",
  custom: "Personalizada",
};

export default function Settings() {
  const [style, setStyle] = useState<Style>("balanced");
  const [custom, setCustom] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [follow, setFollow] = useState(false);
  const [model, setModel] = useState("sonnet");

  useEffect(() => {
    api.getSetting("cover_letter_style").then((v) => { if (v) setStyle(v as Style); });
    api.getSetting("cover_letter_custom").then((v) => { if (v) setCustom(v); });
    api.getSetting("follow_company").then((v) => { setFollow(v === "true"); });
    api.getSetting("agent_model").then((v) => { if (v) setModel(v); });
  }, []);

  const handleModelChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setModel(value);
    await api.setSetting("agent_model", value);
  };

  async function save() {
    setStatus(null);
    try {
      await api.setSetting("cover_letter_style", style);
      await api.setSetting("cover_letter_custom", custom);
      setStatus("Configurações salvas — valem para as próximas buscas.");
    } catch (e) {
      setStatus(`Erro ao salvar: ${e}`);
    }
  }

  return (
    <section>
      <h1>Configurações</h1>

      <div className="card">
        <h2>Estilo da carta de apresentação</h2>
        <p className="hint">Como o agente escreve a carta em cada candidatura (modo Revisar).</p>
        <label className="field" style={{ marginTop: 14 }}>
          Estilo
          <select value={style} onChange={(e) => setStyle(e.target.value as Style)}>
            {(Object.keys(LABELS) as Style[]).map((s) => (
              <option key={s} value={s}>{LABELS[s]}</option>
            ))}
          </select>
        </label>
        {style === "custom" && (
          <label className="field">
            Suas instruções
            <textarea
              rows={5}
              value={custom}
              onChange={(e) => setCustom(e.target.value)}
              placeholder="Ex.: 2 parágrafos, tom informal, em português, foco em impacto e números, sem jargão."
            />
          </label>
        )}
        <div style={{ display: "flex", gap: 12, alignItems: "center", marginTop: 8 }}>
          <button className="btn btn-primary" onClick={save}>Salvar</button>
          {status && <span className="hint">{status}</span>}
        </div>
      </div>
      <div className="card">
        <h2>Candidatura</h2>
        <p className="hint">Por padrão o agente não segue a empresa.</p>
        <label className="field" style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
          <input type="checkbox" style={{ width: "auto" }} checked={follow}
            onChange={async (e) => { setFollow(e.target.checked); await api.setSetting("follow_company", e.target.checked ? "true" : "false"); }} />
          Seguir a empresa ao se candidatar
        </label>
      </div>

      <div className="card">
        <h3>Agente</h3>
        <div className="field">
          <label>Modelo do agente</label>
          <select value={model} onChange={handleModelChange}>
            <option value="opus">Opus 4.8 (melhor qualidade)</option>
            <option value="sonnet">Sonnet 4.6 (rápido — recomendado)</option>
            <option value="haiku">Haiku 4.5 (mais rápido, menos confiável)</option>
          </select>
          <p className="hint">Aplica-se à busca, envio e análise de currículo.</p>
        </div>
      </div>
    </section>
  );
}
