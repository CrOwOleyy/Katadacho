import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles/globals.css";
import { IslandApp } from "./island/IslandApp";
import { SettingsProvider } from "./store/SettingsContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsProvider>
      <IslandApp />
    </SettingsProvider>
  </React.StrictMode>,
);
