import React, { useState } from "react";
import { Box, Text, useInput } from "ink";
import TextInput from "ink-text-input";
import Spinner from "ink-spinner";
import type { Config } from "../utils/config.ts";
import { testCredentials } from "../utils/untis.ts";

interface LoginProps {
  onLogin: (config: Config) => void;
  initialConfig?: Config | null;
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

export default function Login({ onLogin, initialConfig }: LoginProps) {
  const [values, setValues] = useState<Record<Field, string>>({
    school: initialConfig?.school || "",
    username: initialConfig?.username || "",
    password: initialConfig?.password || "",
    server: initialConfig?.server || "",
  });
  const [activeField, setActiveField] = useState(0);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useInput((_input, key) => {
    if (loading) return;

    if (key.upArrow) {
      setActiveField((prev) => Math.max(0, prev - 1));
    }
    if (key.downArrow) {
      setActiveField((prev) => Math.min(FIELDS.length - 1, prev + 1));
    }
  });

  const handleSubmit = async () => {
    const config: Config = {
      school: values.school,
      username: values.username,
      password: values.password,
      server: values.server,
    };

    if (!config.server || !config.school || !config.username || !config.password) {
      setError("All fields are required");
      return;
    }

    setLoading(true);
    setError("");

    const success = await testCredentials(config);
    if (success) {
      onLogin(config);
    } else {
      setError("Login failed. Check your credentials and try again.");
      setLoading(false);
    }
  };

  return (
    <Box flexDirection="column" padding={1}>
      <Box marginBottom={1}>
        <Text bold color="cyan">
          WebUntis TUI - Login
        </Text>
      </Box>

      <Box marginBottom={1}>
        <Text dimColor>
          Enter your WebUntis credentials. Use Tab/Enter to navigate fields.
        </Text>
      </Box>

      {FIELDS.map((field, index) => (
        <Box key={field.key} marginBottom={0}>
          <Box width={12}>
            <Text
              color={index === activeField ? "cyan" : "white"}
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
                mask={field.key === "password" ? "*" : undefined}
                focus={true}
              />
            ) : (
              <Text dimColor={index !== activeField}>
                {field.key === "password"
                  ? "*".repeat(values[field.key].length) || field.placeholder
                  : values[field.key] || field.placeholder}
              </Text>
            )}
          </Box>
        </Box>
      ))}

      {loading && (
        <Box marginTop={1}>
          <Text color="yellow">
            <Spinner type="dots" />
          </Text>
          <Text color="yellow"> Authenticating...</Text>
        </Box>
      )}

      {error && (
        <Box marginTop={1}>
          <Text color="red">{error}</Text>
        </Box>
      )}

      {!loading && (
        <Box marginTop={1}>
          <Text dimColor>
            Press Enter to move to the next field. Submit on the last field to
            log in.
          </Text>
        </Box>
      )}
    </Box>
  );
}
