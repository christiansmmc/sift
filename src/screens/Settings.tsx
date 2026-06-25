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
      <div className="config-header">
        <h1 className="config-title">Configurações</h1>
        <p className="config-subtitle">Ajuste como o agente escreve, se candidata e qual modelo usa.</p>
      </div>

      <div className="config">
        {/* Card 1: Cover-letter style */}
        <div className="card">
          <div className="config-card-title">Estilo da carta de apresentação</div>
          <div className="config-card-desc">Como o agente escreve a carta em cada candidatura (modo Revisar).</div>
          <div className="config-select-row">
            <div className="config-select-wrap">
              <select value={style} onChange={(e) => setStyle(e.target.value as Style)}>
                {(Object.keys(LABELS) as Style[]).map((s) => (
                  <option key={s} value={s}>{LABELS[s]}</option>
                ))}
              </select>
              <span className="config-chevron">▾</span>
            </div>
            <button className="btn btn-primary" onClick={save}>Salvar</button>
          </div>
          {style === "custom" && (
            <label className="field" style={{ marginTop: 12 }}>
              Suas instruções
              <textarea
                rows={5}
                value={custom}
                onChange={(e) => setCustom(e.target.value)}
                placeholder="Ex.: 2 parágrafos, tom informal, em português, foco em impacto e números, sem jargão."
              />
            </label>
          )}
          {status && <p className="hint" style={{ marginTop: 8 }}>{status}</p>}
        </div>

        {/* Card 2: Follow company */}
        <div className="card">
          <div className="config-card-title">Candidatura</div>
          <div className="config-card-desc">Por padrão o agente não segue a empresa.</div>
          <label className="config-follow-row">
            <input
              type="checkbox"
              checked={follow}
              onChange={async (e) => {
                setFollow(e.target.checked);
                await api.setSetting("follow_company", e.target.checked ? "true" : "false");
              }}
            />
            Seguir a empresa ao se candidatar
          </label>
        </div>

        {/* Card 3: Agent model */}
        <div className="card">
          <div className="config-card-title">Agente</div>
          <div className="config-card-desc">Aplica-se à busca, ao envio e à análise de currículo.</div>
          <div className="config-model-row">
            <span className="config-model-label">Modelo do agente</span>
            <div className="config-select-wrap">
              <select value={model} onChange={handleModelChange}>
                <option value="opus">Opus 4.8 (melhor qualidade)</option>
                <option value="sonnet">Sonnet 4.6 (rápido — recomendado)</option>
                <option value="haiku">Haiku 4.5 (mais rápido, menos confiável)</option>
              </select>
              <span className="config-chevron">▾</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
