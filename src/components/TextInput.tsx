import React, { useCallback, useEffect, useRef, useState } from "react";
import { Text } from "ink";
import { useStableInput } from "./useStableInput.ts";

interface TextInputProps {
  value: string;
  placeholder?: string;
  focus?: boolean;
  mask?: string;
  showCursor?: boolean;
  onChange: (value: string) => void;
  onSubmit?: (value: string) => void;
  onKey?: (input: string, key: TextInputKey) => boolean | void;
}

interface TextInputKey {
  upArrow: boolean;
  downArrow: boolean;
  leftArrow: boolean;
  rightArrow: boolean;
  pageDown: boolean;
  pageUp: boolean;
  home: boolean;
  end: boolean;
  return: boolean;
  escape: boolean;
  ctrl: boolean;
  shift: boolean;
  tab: boolean;
  backspace: boolean;
  delete: boolean;
  meta: boolean;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

export default function TextInput({
  value,
  placeholder = "",
  focus = true,
  mask,
  showCursor = true,
  onChange,
  onSubmit,
  onKey,
}: TextInputProps) {
  const [inputValue, setInputValue] = useState(value);
  const [cursorOffset, setCursorOffset] = useState(value.length);
  const valueRef = useRef(value);
  const cursorRef = useRef(cursorOffset);
  const pendingAcksRef = useRef<string[]>([]);
  const onChangeRef = useRef(onChange);
  const onSubmitRef = useRef(onSubmit);
  const onKeyRef = useRef(onKey);
  const showCursorRef = useRef(showCursor);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    onSubmitRef.current = onSubmit;
  }, [onSubmit]);

  useEffect(() => {
    onKeyRef.current = onKey;
  }, [onKey]);

  useEffect(() => {
    showCursorRef.current = showCursor;
  }, [showCursor]);

  useEffect(() => {
    const pending = pendingAcksRef.current;
    const ackIndex = pending.indexOf(value);

    if (ackIndex !== -1) {
      pendingAcksRef.current = pending.slice(ackIndex + 1);
      return;
    }

    if (value !== valueRef.current) {
      pendingAcksRef.current = [];
      valueRef.current = value;
      setInputValue(value);

      const nextCursor = clamp(cursorRef.current, 0, value.length);
      if (nextCursor !== cursorRef.current) {
        cursorRef.current = nextCursor;
        setCursorOffset(nextCursor);
      }
    }
  }, [value]);

  const applyTextChunk = (
    currentValue: string,
    currentCursor: number,
    chunk: string,
  ): { nextValue: string; nextCursor: number } => {
    let nextValue = currentValue;
    let nextCursor = currentCursor;

    for (const char of chunk) {
      const code = char.codePointAt(0) ?? -1;

      // Treat raw DEL/BS as backspace operations when they arrive in a batch.
      if (code === 127 || code === 8) {
        if (nextCursor > 0) {
          nextValue =
            nextValue.slice(0, nextCursor - 1) + nextValue.slice(nextCursor);
          nextCursor -= 1;
        }
        continue;
      }

      // Ignore remaining control characters.
      if (code < 32 || code === 0x7f) {
        continue;
      }

      nextValue =
        nextValue.slice(0, nextCursor) + char + nextValue.slice(nextCursor);
      nextCursor += char.length;
    }

    return { nextValue, nextCursor };
  };

  const handleInput = useCallback((input: string, key: TextInputKey) => {
    if (onKeyRef.current?.(input, key)) {
      return;
    }

    if (
      key.upArrow ||
      key.downArrow ||
      (key.ctrl && input === "c") ||
      key.tab ||
      (key.shift && key.tab)
    ) {
      return;
    }

    const currentValue = valueRef.current;
    const currentCursor = cursorRef.current;
    let nextValue = currentValue;
    let nextCursor = currentCursor;

    if (key.return) {
      onSubmitRef.current?.(currentValue);
      return;
    }

    if (key.leftArrow) {
      if (showCursorRef.current) {
        nextCursor -= 1;
      }
    } else if (key.rightArrow) {
      if (showCursorRef.current) {
        nextCursor += 1;
      }
    } else if (key.backspace || key.delete) {
      if (currentCursor > 0) {
        nextValue =
          currentValue.slice(0, currentCursor - 1) +
          currentValue.slice(currentCursor);
        nextCursor -= 1;
      }
    } else if (input.length > 0) {
      const applied = applyTextChunk(currentValue, currentCursor, input);
      nextValue = applied.nextValue;
      nextCursor = applied.nextCursor;
    }

    nextCursor = clamp(nextCursor, 0, nextValue.length);

    if (nextCursor !== currentCursor) {
      cursorRef.current = nextCursor;
      setCursorOffset(nextCursor);
    }

    if (nextValue !== currentValue) {
      valueRef.current = nextValue;
      setInputValue(nextValue);
      pendingAcksRef.current = [...pendingAcksRef.current, nextValue];
      onChangeRef.current(nextValue);
    }
  }, []);

  useStableInput(handleInput, { isActive: focus });

  const maskedValue = mask ? mask.repeat(inputValue.length) : inputValue;

  if (!focus || !showCursor) {
    if (placeholder && maskedValue.length === 0) {
      return <Text dimColor>{placeholder}</Text>;
    }

    return <Text>{maskedValue}</Text>;
  }

  if (maskedValue.length === 0) {
    if (!placeholder) {
      return (
        <Text>
          <Text inverse>{" "}</Text>
        </Text>
      );
    }

    return (
      <Text>
        <Text inverse>{placeholder[0] ?? " "}</Text>
        <Text dimColor>{placeholder.slice(1)}</Text>
      </Text>
    );
  }

  const cursor = clamp(cursorOffset, 0, maskedValue.length);
  const before = maskedValue.slice(0, cursor);
  const cursorChar = cursor < maskedValue.length ? maskedValue[cursor] : " ";
  const after = cursor < maskedValue.length ? maskedValue.slice(cursor + 1) : "";

  return (
    <Text>
      {before}
      <Text inverse>{cursorChar}</Text>
      {after}
    </Text>
  );
}
