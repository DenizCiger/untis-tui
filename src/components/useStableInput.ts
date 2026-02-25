import { useInput, type Key } from "ink";
import { useCallback, useEffect, useRef } from "react";

type InputHandler = (input: string, key: Key) => void;

interface InputOptions {
  isActive?: boolean;
}

export function useStableInput(handler: InputHandler, options?: InputOptions) {
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  const stableHandler = useCallback<InputHandler>((input, key) => {
    handlerRef.current(input, key);
  }, []);

  useInput(stableHandler, options);
}
