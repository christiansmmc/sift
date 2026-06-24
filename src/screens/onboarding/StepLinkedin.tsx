type Login = { username: string; password: string };

export default function StepLinkedin({
  value, onChange,
}: { value: Login; onChange: (v: Login) => void }) {
  return (
    <section className="step">
      <h2>Login LinkedIn</h2>
      <p className="hint">Guardado com segurança no Gerenciador de Credenciais do Windows — nunca em texto puro.</p>
      <label>E-mail / usuário
        <input value={value.username} onChange={(e) => onChange({ ...value, username: e.target.value })} />
      </label>
      <label>Senha
        <input type="password" value={value.password} onChange={(e) => onChange({ ...value, password: e.target.value })} />
      </label>
    </section>
  );
}
