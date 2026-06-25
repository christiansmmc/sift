import { useEffect, useState } from "react";
import { api, onAgentEvent, onAgentStatus } from "./lib/api";
import { getTheme, toggleTheme, type Theme } from "./lib/theme";
import type { DashboardCounts } from "./types";
import Onboarding from "./screens/Onboarding";
import Dashboard from "./screens/Dashboard";
import Jobs from "./screens/Jobs";
import Pending from "./screens/Pending";
import Profile from "./screens/Profile";
import Settings from "./screens/Settings";

type Screen = "dashboard" | "jobs" | "pending" | "profile" | "settings";

const NAV: { key: Screen; label: string }[] = [
  { key: "dashboard", label: "Painel" },
  { key: "jobs", label: "Vagas" },
  { key: "pending", label: "Pendências" },
  { key: "profile", label: "Perfil" },
  { key: "settings", label: "Configurações" },
];

export default function App() {
  const [onboarded, setOnboarded] = useState<boolean | null>(null);
  const [screen, setScreen] = useState<Screen>("dashboard");
  const [theme, setThemeState] = useState<Theme>(getTheme());

  // Agent/dashboard state lives here — App is always mounted, so it survives
  // tab switches (the Painel is conditionally rendered) and the live feed keeps
  // accumulating even while the user is on another screen.
  const [counts, setCounts] = useState<DashboardCounts | null>(null);
  const [running, setRunning] = useState(false);
  const [mode, setMode] = useState<"scan" | "revisar">("revisar");
  const [batch, setBatch] = useState(10);
  const [feed, setFeed] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [approvedCount, setApprovedCount] = useState(0);

  useEffect(() => {
    api.getOnboardingStatus().then(setOnboarded).catch(() => setOnboarded(false));
  }, []);

  async function refreshDashboard() {
    setCounts(await api.dashboardCounts());
    setRunning(await api.agentRunning());
    setApprovedCount(await api.countApproved());
  }

  // Subscribe once (after onboarding) for the app's lifetime.
  useEffect(() => {
    if (!onboarded) return;
    refreshDashboard();
    const un = onAgentEvent(() => refreshDashboard());
    const unStatus = onAgentStatus((t) => setFeed((f) => [...f, t].slice(-50)));
    return () => {
      un.then((f) => f());
      unStatus.then((f) => f());
    };
  }, [onboarded]);

  async function onStart() {
    setError(null);
    setFeed([]);
    try {
      await api.startSearchBatch(mode, batch);
      setRunning(true);
    } catch (e) {
      setError(String(e));
    }
  }
  async function onStop() {
    await api.stopAgent();
    setRunning(false);
  }
  async function onSubmitApproved() {
    setError(null);
    setFeed([]);
    try { await api.submitApproved(); setRunning(true); }
    catch (e) { setError(String(e)); }
  }

  if (onboarded === null) return <div className="loading">Carregando…</div>;
  if (!onboarded) return <Onboarding onDone={() => setOnboarded(true)} />;

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="brand">apply<span>bot</span></div>
        <nav>
          {NAV.map((n) => (
            <button
              key={n.key}
              className={`navlink ${screen === n.key ? "active" : ""}`}
              onClick={() => setScreen(n.key)}
            >
              {n.label}
            </button>
          ))}
        </nav>
        <button className="theme-toggle" onClick={() => setThemeState(toggleTheme())}>
          {theme === "dark" ? "☀️  Tema claro" : "🌙  Tema escuro"}
        </button>
      </nav>
      <main className="content">
        {screen === "dashboard" && (
          <Dashboard
            counts={counts}
            running={running}
            mode={mode}
            batch={batch}
            feed={feed}
            error={error}
            setMode={setMode}
            setBatch={setBatch}
            onStart={onStart}
            onStop={onStop}
            approvedCount={approvedCount}
            onSubmitApproved={onSubmitApproved}
          />
        )}
        {screen === "jobs" && <Jobs />}
        {screen === "pending" && <Pending />}
        {screen === "profile" && <Profile />}
        {screen === "settings" && <Settings />}
      </main>
    </div>
  );
}
