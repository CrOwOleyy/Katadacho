import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "./styles/globals.css";
import { Ghost } from "./components/mascot/Ghost";
import { GHOST_MOODS } from "./components/mascot/mood";

/** Page de développement : toutes les humeurs de la mascotte, à plusieurs tailles. */
function Preview() {
  return (
    <div style={{ background: "#08080a", color: "#f4f4f5", minHeight: "100vh", padding: 24, overflow: "auto", userSelect: "auto" }}>
      <h1 style={{ fontSize: 14, opacity: 0.6, marginBottom: 16 }}>Mascotte : humeurs et tailles</h1>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 16 }}>
        {GHOST_MOODS.map((mood) => (
          <div key={mood} style={{ background: "#121216", borderRadius: 16, padding: 16, textAlign: "center" }}>
            <Ghost size={150} mood={mood} />
            <p style={{ fontSize: 12, opacity: 0.6, margin: "8px 0" }}>{mood}</p>
            <div style={{ display: "flex", justifyContent: "center", alignItems: "flex-end", gap: 14 }}>
              <Ghost size={64} mood={mood} />
              <Ghost size={30} mood={mood} />
              <Ghost size={22} mood={mood} />
            </div>
          </div>
        ))}
      </div>

      <h2 style={{ fontSize: 14, opacity: 0.6, margin: "24px 0 8px" }}>Dans la pilule (36 px de haut)</h2>
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "space-between",
          width: 270,
          height: 36,
          padding: "0 12px",
          borderRadius: 999,
          background: "#08080a",
          border: "1px solid rgba(255,255,255,0.1)",
        }}
      >
        <Ghost size={30} mood="music" />
        <span style={{ fontFamily: "monospace", fontSize: 12, opacity: 0.7 }}>19:12</span>
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Preview />
  </React.StrictMode>,
);
