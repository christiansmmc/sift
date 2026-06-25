import { useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile } from "../types";
import StepPersonal from "./onboarding/StepPersonal";
import StepCv from "./onboarding/StepCv";
import StepCriteria from "./onboarding/StepCriteria";

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
  const isDone = step === 3;

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
      setStep(3);
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  const canFinish =
    personal.full_name.trim() !== "" &&
    cvText.trim() !== "" &&
    criteria.role.trim() !== "";

  const progressPct = Math.round((Math.min(step, 2) / 2) * 100);

  return (
    <div className="onb-wrap">
      <div className="onb">
        {!isDone ? (
          <>
            <header className="onb-head">
              <h1 className="onb-title">Configuração inicial</h1>
              <div className="onb-steps">
                {steps.map((s, i) => (
                  <button
                    key={s}
                    className={`onb-step${i === step ? " onb-step--active" : i < step ? " onb-step--done" : ""}`}
                    onClick={() => { if (i < step) setStep(i); }}
                    disabled={i >= step}
                  >
                    {s}
                  </button>
                ))}
              </div>
              <div className="onb-progress">
                <div className="onb-progress-fill" style={{ width: `${progressPct}%` }} />
              </div>
            </header>
            <div className="onb-body">
              {step === 0 && <StepCv cvText={cvText} setCvText={setCvText} personal={personal} setPersonal={setPersonal} criteria={criteria} setCriteria={setCriteria} />}
              {step === 1 && <StepPersonal value={personal} onChange={setPersonal} />}
              {step === 2 && <StepCriteria value={criteria} onChange={setCriteria} />}
            </div>
            <footer className="onb-foot">
              {error && <p className="onb-error">{error}</p>}
              {!error && step === 1 && !personal.full_name.trim() && (
                <p className="onb-error">O nome é obrigatório para concluir.</p>
              )}
              {!error && step === 2 && !criteria.role.trim() && (
                <p className="onb-error">O cargo é obrigatório para concluir.</p>
              )}
              <div className="onb-foot-btns">
                <button className="btn" disabled={step === 0 || saving} onClick={() => setStep((s) => s - 1)}>Voltar</button>
                {step < 2 ? (
                  <button className="btn btn-primary" onClick={() => setStep((s) => s + 1)}>Próximo</button>
                ) : (
                  <button className="btn btn-primary" disabled={!canFinish || saving} onClick={finish}>
                    {saving ? "Salvando…" : "Concluir"}
                  </button>
                )}
              </div>
            </footer>
          </>
        ) : (
          <div className="onb-done">
            <div className="onb-done-icon">
              <svg width="30" height="30" viewBox="0 0 24 24">
                <polyline points="5,12.5 10,17.5 19,7" fill="none" stroke="var(--ok)" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </div>
            <h1 className="onb-done-title">Tudo pronto!</h1>
            <p className="onb-done-body">Seu perfil e critérios de busca estão configurados. O applybot já pode começar a encontrar vagas para você.</p>
            <div className="onb-done-actions">
              <button className="btn" onClick={() => setStep(0)}>Revisar novamente</button>
              <button className="btn btn-primary" onClick={onDone}>Começar a usar o applybot</button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
