import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initializeI18n } from "./localization/i18n";
import "./styles.css";

async function bootstrap(): Promise<void> {
  await initializeI18n();

  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void bootstrap();
