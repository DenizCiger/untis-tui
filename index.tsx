import React from "react";
import { render } from "ink";
import App from "./src/components/App.tsx";

// Enable raw mode with error handling for unsupported environments
try {
  process.stdin.setRawMode?.(true);
} catch (e) {
  // Raw mode not supported, will continue anyway
}

render(<App />);
