export type Theme = "light" | "dark";
const KEY = "applybot-theme";

export function getTheme(): Theme {
  return (localStorage.getItem(KEY) as Theme) ?? "dark";
}
export function setTheme(t: Theme) {
  localStorage.setItem(KEY, t);
  document.documentElement.dataset.theme = t;
}
export function toggleTheme(): Theme {
  const next: Theme = getTheme() === "dark" ? "light" : "dark";
  setTheme(next);
  return next;
}
export function initTheme() {
  setTheme(getTheme());
}
