import React from "react";
import ReactDOM from "react-dom/client";
import "./styles/index.css";
import { initTheme } from "./lib/theme";
import App from "./App";

initTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
