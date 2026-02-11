import React, { useState, useEffect } from "react";
import { Box, Text } from "ink";
import type { Config, SavedConfig } from "../utils/config.ts";
import { loadConfig, saveConfig, clearConfig } from "../utils/config.ts";
import { clearCache } from "../utils/cache.ts";
import {
  clearPassword,
  getSecureStorageDiagnostic,
  loadPassword,
  savePassword,
} from "../utils/secret.ts";
import Login from "./Login.tsx";
import Timetable from "./Timetable.tsx";

type Screen = "loading" | "login" | "timetable";

export default function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [savedConfig, setSavedConfig] = useState<SavedConfig | null>(null);
  const [config, setConfig] = useState<Config | null>(null);
  const [error, setError] = useState("");
  const [secureStorageNotice, setSecureStorageNotice] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function init() {
      const storage = getSecureStorageDiagnostic();
      if (!cancelled) {
        setSecureStorageNotice(storage.available ? "" : storage.message);
      }

      const saved = loadConfig();
      if (!saved) {
        if (!cancelled) {
          setScreen("login");
        }
        return;
      }

      if (cancelled) return;
      setSavedConfig(saved);

      const password = await loadPassword(saved);
      if (cancelled) return;

      if (password) {
        setConfig({ ...saved, password });
        setScreen("timetable");
      } else {
        setScreen("login");
      }
    }

    void init();

    return () => {
      cancelled = true;
    };
  }, []);

  const handleLogin = async (newConfig: Config) => {
    const nextSavedConfig: SavedConfig = {
      school: newConfig.school,
      username: newConfig.username,
      server: newConfig.server,
    };

    try {
      saveConfig(newConfig);
      setSavedConfig(nextSavedConfig);
      setError("");
    } catch {
      setError("Login succeeded, but profile settings could not be saved to disk.");
    }

    try {
      await savePassword(nextSavedConfig, newConfig.password);
    } catch {
      setError("Login succeeded, but secure password storage failed. You will need to log in again next time.");
    }

    setConfig(newConfig);
    setScreen("timetable");
  };

  const handleLogout = () => {
    const activeProfile =
      config
        ? {
            school: config.school,
            username: config.username,
            server: config.server,
          }
        : savedConfig;

    if (activeProfile) {
      void clearPassword(activeProfile);
    }

    clearConfig();
    clearCache();
    setSavedConfig(null);
    setError("");
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
    return (
      <Login
        onLogin={handleLogin}
        initialConfig={savedConfig}
        error={error}
        secureStorageNotice={secureStorageNotice}
      />
    );
  }

  if (screen === "timetable" && config) {
    return <Timetable config={config} onLogout={handleLogout} />;
  }

  return null;
}
