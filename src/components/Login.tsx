import React, { useState } from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import TextInput from "./TextInput.tsx";
import type { Config, SavedConfig } from "../utils/config.ts";
import { testCredentials } from "../utils/untis.ts";
import { useStableInput } from "./useStableInput.ts";

interface LoginProps {
  onLogin: (config: Config) => Promise<void> | void;
  initialConfig?: SavedConfig | null;
  savedLoginConfig?: Config | null;
  error?: string;
  secureStorageNotice?: string;
}

type Field = "school" | "username" | "password" | "server";

const FIELDS: { key: Field; label: string; placeholder: string }[] = [
  {
    key: "server",
    label: "Server",
    placeholder: "e.g. mese.webuntis.com",
  },
  {
    key: "school",
    label: "School",
    placeholder: "Your school name from the URL",
  },
  { key: "username", label: "Username", placeholder: "Your WebUntis username" },
  { key: "password", label: "Password", placeholder: "Your WebUntis password" },
];

export default function Login({
  onLogin,
  initialConfig,
  savedLoginConfig,
  error: appError,
  secureStorageNotice,
}: LoginProps) {
  const [values, setValues] = useState<Record<Field, string>>({
    school: initialConfig?.school || "",
    username: initialConfig?.username || "",
    password: "",
    server: initialConfig?.server || "",
  });
  const [activeField, setActiveField] = useState(0);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);

  const authenticateAndLogin = async (
    config: Config,
    failureMessage: string,
  ) => {
    setLoading(true);
    setError("");

    try {
      const success = await testCredentials(config);
      if (!success) {
        setError(failureMessage);
        setLoading(false);
        return;
      }

      await onLogin(config);
    } catch {
      setError(failureMessage);
      setLoading(false);
    }
  };

  const handleSubmit = async () => {
    const config: Config = {
      school: values.school.trim(),
      username: values.username.trim(),
      password: values.password,
      server: values.server.trim(),
    };

    if (!config.server || !config.school || !config.username || !config.password) {
      setError("All fields are required");
      return;
    }

    await authenticateAndLogin(
      config,
      "Login failed. Check your credentials and try again.",
    );
  };

  const handleSavedLogin = async () => {
    if (!savedLoginConfig || loading) {
      return;
    }

    await authenticateAndLogin(
      savedLoginConfig,
      "Saved login failed. Re-enter your credentials and try again.",
    );
  };

  useStableInput(
    (_input, key) => {
      if (loading) return;

      if (key.tab && key.shift) {
        setActiveField((prev) => Math.max(0, prev - 1));
        return;
      }
      if (key.tab) {
        setActiveField((prev) => Math.min(FIELDS.length - 1, prev + 1));
        return;
      }
      if (key.upArrow) {
        setActiveField((prev) => Math.max(0, prev - 1));
      }
      if (key.downArrow) {
        setActiveField((prev) => Math.min(FIELDS.length - 1, prev + 1));
      }

      if (key.ctrl && _input === "v") {
        setShowPassword((prev) => !prev);
      }

      if (key.ctrl && _input === "l" && savedLoginConfig) {
        void handleSavedLogin();
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  return (
    <Box flexDirection="column" padding={1}>
      <Box marginBottom={1}>
        <Text bold color={COLORS.brand}>
          WebUntis TUI - Login
        </Text>
      </Box>

      <Box marginBottom={1}>
        <Text dimColor>
          Enter your WebUntis credentials. Use arrows or Tab to change focus.
        </Text>
      </Box>

      <Box marginBottom={1}>
        <Text dimColor>Password is stored securely via your OS credentials store.</Text>
      </Box>

      {savedLoginConfig && (
        <Box marginBottom={1}>
          <Text color={COLORS.brand}>
            {`Saved account: ${savedLoginConfig.username}@${savedLoginConfig.school} (${savedLoginConfig.server}) | Press Ctrl+l to login with saved credentials`}
          </Text>
        </Box>
      )}

      {FIELDS.map((field, index) => (
        <Box key={field.key} marginBottom={0}>
          <Box width={12}>
            <Text
              color={index === activeField ? COLORS.brand : COLORS.neutral.white}
              bold={index === activeField}
            >
              {index === activeField ? "> " : "  "}
              {field.label}:
            </Text>
          </Box>
          <Box marginLeft={1}>
            {index === activeField && !loading ? (
              <TextInput
                value={values[field.key]}
                onChange={(val) =>
                  setValues((prev) => ({ ...prev, [field.key]: val }))
                }
                onSubmit={() => {
                  if (activeField < FIELDS.length - 1) {
                    setActiveField(activeField + 1);
                  } else {
                    handleSubmit();
                  }
                }}
                placeholder={field.placeholder}
                mask={field.key === "password" && !showPassword ? "*" : undefined}
                focus={true}
              />
            ) : (
              <Text dimColor={index !== activeField}>
                {field.key === "password"
                  ? showPassword
                    ? values[field.key] || field.placeholder
                    : "*".repeat(values[field.key].length) || field.placeholder
                  : values[field.key] || field.placeholder}
              </Text>
            )}
          </Box>
        </Box>
      ))}

      {loading && (
        <Box marginTop={1}>
          <Text color={COLORS.warning}>
            <Spinner type="dots" />
          </Text>
          <Text color={COLORS.warning}> Authenticating...</Text>
        </Box>
      )}

      {(appError || error) && (
        <Box marginTop={1}>
          <Text color={COLORS.error}>{appError || error}</Text>
        </Box>
      )}

      {secureStorageNotice && (
        <Box marginTop={1}>
          <Text color={COLORS.warning}>{secureStorageNotice}</Text>
        </Box>
      )}

      {!loading && (
        <Box marginTop={1}>
          <Text dimColor>
            {savedLoginConfig
              ? "Enter next/submit | Tab move focus | Ctrl+v toggle password visibility | Ctrl+l login with saved account"
              : "Enter next/submit | Tab move focus | Ctrl+v toggle password visibility"}
          </Text>
        </Box>
      )}
    </Box>
  );
}
