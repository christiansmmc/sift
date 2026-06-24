import { useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile } from "../types";
import StepPersonal from "./onboarding/StepPersonal";
import StepCv from "./onboarding/StepCv";
import StepCriteria from "./onboarding/StepCriteria";
import StepLinkedin from "./onboarding/StepLinkedin";
import "../onboarding.css";

const EMPTY_CRITERIA: Criteria = {
  role: "", seniority: "", work_model: "", locations: [], salary_min: null, red_lines: [],
};

export default function Onboarding({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [personal, setPersonal] = useState({ full_name: "", email: "", phone: "", location: "" });
  const [cvText, setCvText] = useState("");
  const [criteria, setCriteria] = useState<Criteria>(EMPTY_CRITERIA);
  const [linkedin, setLinkedin] = useState({ username: "", password: "" });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const steps = ["Seus dados", "Currículo", "O que você busca", "Login LinkedIn"];

  async function finish() {
    setSaving(true);
    setError(null);
    try {
      const profile: Profile = {
        ...personal,
        cv_text: cvText,
        criteria_json: JSON.stringify(criteria),
      };
      await api.saveProfile(profile);
      await api.saveLinkedinCredentials(linkedin.username, linkedin.password);
      onDone();
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  const canFinish =
    personal.full_name.trim() !== "" &&
    cvText.trim() !== "" &&
    criteria.role.trim() !== "" &&
    linkedin.username.trim() !== "" &&
    linkedin.password.trim() !== "";

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
        {step === 0 && <StepPersonal value={personal} onChange={setPersonal} />}
        {step === 1 && <StepCv cvText={cvText} setCvText={setCvText} criteria={criteria} setCriteria={setCriteria} />}
        {step === 2 && <StepCriteria value={criteria} onChange={setCriteria} />}
        {step === 3 && <StepLinkedin value={linkedin} onChange={setLinkedin} />}
      </main>
      {error && <p className="onb-error">Erro ao salvar: {error}</p>}
      <footer className="onb-foot">
        <button disabled={step === 0 || saving} onClick={() => setStep((s) => s - 1)}>Voltar</button>
        {step < 3 ? (
          <button onClick={() => setStep((s) => s + 1)}>Próximo</button>
        ) : (
          <button disabled={!canFinish || saving} onClick={finish}>
            {saving ? "Salvando…" : "Concluir"}
          </button>
        )}
      </footer>
    </div>
  );
}
