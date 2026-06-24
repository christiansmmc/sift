import type { Criteria } from "../../types";

export default function StepCriteria({
  value, onChange,
}: { value: Criteria; onChange: (c: Criteria) => void }) {
  const set = <K extends keyof Criteria,>(k: K, v: Criteria[K]) => onChange({ ...value, [k]: v });
  return (
    <section className="step">
      <h2>O que você busca</h2>
      <p className="hint">Pré-preenchido pela análise do currículo quando disponível. Ajuste à vontade.</p>
      <label>Cargo<input value={value.role} onChange={(e) => set("role", e.target.value)} /></label>
      <label>Senioridade<input value={value.seniority} onChange={(e) => set("seniority", e.target.value)} placeholder="junior / mid / senior / lead" /></label>
      <label>Modelo de trabalho
        <select value={value.work_model} onChange={(e) => set("work_model", e.target.value)}>
          <option value="">Indiferente</option>
          <option value="remote">Remoto</option>
          <option value="hybrid">Híbrido</option>
          <option value="onsite">Presencial</option>
        </select>
      </label>
      <label>Localizações (separadas por vírgula)
        <input value={value.locations.join(", ")}
          onChange={(e) => set("locations", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <label>Salário mínimo (R$)
        <input type="number" value={value.salary_min ?? ""}
          onChange={(e) => set("salary_min", e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>Red-lines (o que evitar, separadas por vírgula)
        <input value={value.red_lines.join(", ")}
          onChange={(e) => set("red_lines", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <p className="hint">O cargo é obrigatório para concluir.</p>
    </section>
  );
}
