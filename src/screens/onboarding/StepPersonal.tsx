type Personal = { full_name: string; email: string; phone: string; location: string };

export default function StepPersonal({
  value, onChange,
}: { value: Personal; onChange: (v: Personal) => void }) {
  const set = (k: keyof Personal) => (e: React.ChangeEvent<HTMLInputElement>) =>
    onChange({ ...value, [k]: e.target.value });
  return (
    <section className="step">
      <div className="onb-eyebrow">Seus dados</div>
      <label className="field">Nome completo<input value={value.full_name} onChange={set("full_name")} /></label>
      <label className="field">E-mail<input value={value.email} onChange={set("email")} /></label>
      <label className="field">Telefone<input value={value.phone} onChange={set("phone")} placeholder="(00) 00000-0000" /></label>
      <label className="field">Localização<input value={value.location} onChange={set("location")} /></label>
    </section>
  );
}
