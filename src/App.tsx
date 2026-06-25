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
import Titlebar from "./Titlebar";

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
  const [runKind, setRunKind] = useState<"search" | "submit">("search");
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

  // Refresh whenever the Painel becomes active, so counters and the
  // "Enviar aprovadas" button reflect actions taken on other tabs (e.g. approving).
  useEffect(() => {
    if (onboarded && screen === "dashboard") refreshDashboard();
  }, [screen, onboarded]);

  async function onStart() {
    setRunKind("search");
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
    setRunKind("submit");
    setError(null);
    setFeed([]);
    try { await api.submitApproved(); setRunning(true); }
    catch (e) { setError(String(e)); }
  }

  if (onboarded === null) return <div className="loading">Carregando…</div>;
  if (!onboarded) return <Onboarding onDone={() => setOnboarded(true)} />;

  return (
    <div className="root">
      <Titlebar />
      <div className="app">
        <nav className="sidebar">
          <div className="brand">apply<span>bot</span></div>
          <div className="sidebar-links">
            {NAV.map((n) => (
              <button
                key={n.key}
                className={`navlink ${screen === n.key ? "active" : ""}`}
                onClick={() => setScreen(n.key)}
              >
                {n.label}
                {n.key === "jobs" && (counts?.awaiting_approval ?? 0) > 0 && (
                  <span className="nav-badge">{counts!.awaiting_approval}</span>
                )}
              </button>
            ))}
          </div>
          <div className="theme-row">
            <span>{theme === "dark" ? "Tema escuro" : "Tema claro"}</span>
            <button
              className={`switch${theme === "dark" ? " on" : ""}`}
              onClick={() => setThemeState(toggleTheme())}
              aria-label="Alternar tema"
            />
          </div>
        </nav>
        <main className="content">
          {screen === "dashboard" && (
            <Dashboard
              counts={counts}
              running={running}
              runKind={runKind}
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
    </div>
  );
}
