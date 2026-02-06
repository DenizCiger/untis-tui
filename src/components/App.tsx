import React, { useState, useEffect } from "react";
import { Box, Text } from "ink";
import type { Config } from "../utils/config.ts";
import { loadConfig, saveConfig, clearConfig } from "../utils/config.ts";
import Login from "./Login.tsx";
import Timetable from "./Timetable.tsx";

type Screen = "loading" | "login" | "timetable";

export default function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [config, setConfig] = useState<Config | null>(null);

  useEffect(() => {
    const saved = loadConfig();
    if (saved) {
      setConfig(saved);
      setScreen("timetable");
    } else {
      setScreen("login");
    }
  }, []);

  const handleLogin = (newConfig: Config) => {
    saveConfig(newConfig);
    setConfig(newConfig);
    setScreen("timetable");
  };

  const handleLogout = () => {
    clearConfig();
    setConfig(null);
    setScreen("login");
  };

  if (screen === "loading") {
    return (
      <Box padding={1}>
        <Text dimColor>Loading...</Text>
      </Box>
    );
  }

  if (screen === "login") {
    return <Login onLogin={handleLogin} initialConfig={config} />;
  }

  if (screen === "timetable" && config) {
    return <Timetable config={config} onLogout={handleLogout} />;
  }

  return null;
}
