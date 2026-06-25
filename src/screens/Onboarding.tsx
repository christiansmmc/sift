import { useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile } from "../types";
import StepPersonal from "./onboarding/StepPersonal";
import StepCv from "./onboarding/StepCv";
import StepCriteria from "./onboarding/StepCriteria";
import "../onboarding.css";

const EMPTY_CRITERIA: Criteria = {
  role: "", seniority: "", work_model: "", locations: [], salary_min: null, red_lines: [],
};

export default function Onboarding({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [personal, setPersonal] = useState({ full_name: "", email: "", phone: "", location: "" });
  const [cvText, setCvText] = useState("");
  const [criteria, setCriteria] = useState<Criteria>(EMPTY_CRITERIA);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const steps = ["Currículo", "Seus dados", "O que você busca"];

  async function finish() {
    setSaving(true);
    setError(null);
    try {
      const profile: Profile = {
        ...personal,
        cv_text: cvText,
        criteria_json: JSON.stringify(criteria),
        screening_json: "{}",
      };
      await api.saveProfile(profile);
      onDone();
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  const canFinish =
    personal.full_name.trim() !== "" &&
    cvText.trim() !== "" &&
    criteria.role.trim() !== "";

  return (
    <div className="onb">
      <header className="onb-head">
        <h1>Configuração inicial</h1>
        <ol className="onb-steps">
          {steps.map((s, i) => (
            <li key={s} className={i === step ? "current" : i < step ? "done" : ""}>{s}</li>
          ))}
        </ol>
      </header>
      <main className="onb-body">
        {step === 0 && <StepCv cvText={cvText} setCvText={setCvText} personal={personal} setPersonal={setPersonal} criteria={criteria} setCriteria={setCriteria} />}
        {step === 1 && <StepPersonal value={personal} onChange={setPersonal} />}
        {step === 2 && <StepCriteria value={criteria} onChange={setCriteria} />}
      </main>
      {error && <p className="onb-error">Erro ao salvar: {error}</p>}
      <footer className="onb-foot">
        <button className="btn" disabled={step === 0 || saving} onClick={() => setStep((s) => s - 1)}>Voltar</button>
        {step < 2 ? (
          <button className="btn btn-primary" onClick={() => setStep((s) => s + 1)}>Próximo</button>
        ) : (
          <button className="btn btn-primary" disabled={!canFinish || saving} onClick={finish}>
            {saving ? "Salvando…" : "Concluir"}
          </button>
        )}
      </footer>
    </div>
  );
}
