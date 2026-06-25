import { useEffect, useState } from "react";
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

  useEffect(() => {
    api.getSetting("cover_letter_style").then((v) => { if (v) setStyle(v as Style); });
    api.getSetting("cover_letter_custom").then((v) => { if (v) setCustom(v); });
  }, []);

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
    </section>
  );
}
