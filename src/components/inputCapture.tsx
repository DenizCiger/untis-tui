import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";

interface InputCaptureContextValue {
  acquire: () => () => void;
}

const InputCaptureContext = createContext<InputCaptureContextValue | null>(null);

interface InputCaptureProviderProps {
  children: React.ReactNode;
  onBlockedChange: (blocked: boolean) => void;
}

export function InputCaptureProvider({ children, onBlockedChange }: InputCaptureProviderProps) {
  const [captureCount, setCaptureCount] = useState(0);

  useEffect(() => {
    onBlockedChange(captureCount > 0);
  }, [captureCount, onBlockedChange]);

  const value = useMemo<InputCaptureContextValue>(
    () => ({
      acquire: () => {
        let released = false;
        setCaptureCount((prev) => prev + 1);

        return () => {
          if (released) return;
          released = true;
          setCaptureCount((prev) => Math.max(0, prev - 1));
        };
      },
    }),
    [],
  );

  return <InputCaptureContext.Provider value={value}>{children}</InputCaptureContext.Provider>;
}

export function useInputCapture(active: boolean) {
  const context = useContext(InputCaptureContext);
  const releaseRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    if (!context) return;

    if (active && !releaseRef.current) {
      releaseRef.current = context.acquire();
    }

    if (!active && releaseRef.current) {
      releaseRef.current();
      releaseRef.current = null;
    }

    return () => {
      if (releaseRef.current) {
        releaseRef.current();
        releaseRef.current = null;
      }
    };
  }, [active, context]);
}
