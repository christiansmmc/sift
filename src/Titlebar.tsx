import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();

export default function Titlebar() {
  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-brand">
        <span className="titlebar-logo" aria-hidden />
        <span className="titlebar-word">applybot</span>
      </div>
      <div className="titlebar-controls">
        <button className="tb-btn" onClick={() => win.minimize()} aria-label="Minimizar">─</button>
        <button className="tb-btn" onClick={() => win.toggleMaximize()} aria-label="Maximizar">▢</button>
        <button className="tb-btn tb-close" onClick={() => win.close()} aria-label="Fechar">✕</button>
      </div>
    </div>
  );
}
