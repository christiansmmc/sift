export default function Onboarding({ onDone }: { onDone: () => void }) {
  return (
    <section style={{ padding: 24 }}>
      <h1>Configuração inicial</h1>
      <p>Vamos configurar seu perfil antes de começar.</p>
      <button onClick={onDone}>Concluir (provisório)</button>
    </section>
  );
}
