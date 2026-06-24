type Personal = { full_name: string; email: string; phone: string; location: string };

export default function StepPersonal({
  value, onChange,
}: { value: Personal; onChange: (v: Personal) => void }) {
  const set = (k: keyof Personal) => (e: React.ChangeEvent<HTMLInputElement>) =>
    onChange({ ...value, [k]: e.target.value });
  return (
    <section className="step">
      <h2>Seus dados</h2>
      <label>Nome completo<input value={value.full_name} onChange={set("full_name")} /></label>
      <label>E-mail<input value={value.email} onChange={set("email")} /></label>
      <label>Telefone<input value={value.phone} onChange={set("phone")} /></label>
      <label>Localização<input value={value.location} onChange={set("location")} /></label>
      <p className="hint">O nome é obrigatório para concluir.</p>
    </section>
  );
}
