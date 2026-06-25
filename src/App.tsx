import { useEffect, useState } from "react";
import { api } from "./lib/api";
import { getTheme, toggleTheme, type Theme } from "./lib/theme";
import Onboarding from "./screens/Onboarding";
import Dashboard from "./screens/Dashboard";
import Jobs from "./screens/Jobs";
import Pending from "./screens/Pending";
import Profile from "./screens/Profile";
import "./App.css";

type Screen = "dashboard" | "jobs" | "pending" | "profile";

const NAV: { key: Screen; label: string }[] = [
  { key: "dashboard", label: "Painel" },
  { key: "jobs", label: "Vagas" },
  { key: "pending", label: "Pendências" },
  { key: "profile", label: "Perfil" },
];

export default function App() {
  const [onboarded, setOnboarded] = useState<boolean | null>(null);
  const [screen, setScreen] = useState<Screen>("dashboard");
  const [theme, setThemeState] = useState<Theme>(getTheme());

  useEffect(() => {
    api.getOnboardingStatus().then(setOnboarded).catch(() => setOnboarded(false));
  }, []);

  if (onboarded === null) return <div className="loading">Carregando…</div>;
  if (!onboarded) return <Onboarding onDone={() => setOnboarded(true)} />;

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="brand">apply<span>bot</span></div>
        <nav>
          {NAV.map((n) => (
            <button key={n.key} className={`navlink ${screen === n.key ? "active" : ""}`} onClick={() => setScreen(n.key)}>
              {n.label}
            </button>
          ))}
        </nav>
        <button className="theme-toggle" onClick={() => setThemeState(toggleTheme())}>
          {theme === "dark" ? "☀️  Tema claro" : "🌙  Tema escuro"}
        </button>
      </nav>
      <main className="content">
        {screen === "dashboard" && <Dashboard />}
        {screen === "jobs" && <Jobs />}
        {screen === "pending" && <Pending />}
        {screen === "profile" && <Profile />}
      </main>
    </div>
  );
}
