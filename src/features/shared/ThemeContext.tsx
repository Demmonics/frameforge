import React, { createContext, useContext, useEffect, useState } from "react";

export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "frameforge.colorScheme";

interface ThemeContextValue {
  preference: ThemePreference;
  systemResolved: ResolvedTheme;
  effectiveTheme: ResolvedTheme;
  setPreference: (pref: ThemePreference) => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  preference: "system",
  systemResolved: "dark",
  effectiveTheme: "dark",
  setPreference: () => {},
});

function readStoredPreference(): ThemePreference {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "light" || raw === "dark") return raw;
  } catch {
    // ignore storage restriction
  }
  return "system";
}

function readSystemDark(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return true;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function syncMetaColorScheme(pref: ThemePreference) {
  const meta = document.querySelector('meta[name="color-scheme"]');
  if (meta) {
    meta.setAttribute("content", pref === "system" ? "light dark" : pref);
  }
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [preference, setPreferenceState] = useState<ThemePreference>(readStoredPreference);
  const [systemDark, setSystemDark] = useState<boolean>(readSystemDark);

  useEffect(() => {
    syncMetaColorScheme(preference);
  }, [preference]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return;
    }
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY) {
        setPreferenceState(readStoredPreference());
      }
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  const setPreference = (next: ThemePreference) => {
    setPreferenceState(next);
    try {
      if (next === "system") {
        localStorage.removeItem(STORAGE_KEY);
      } else {
        localStorage.setItem(STORAGE_KEY, next);
      }
    } catch {
      // ignore storage restriction
    }
    syncMetaColorScheme(next);
  };

  const systemResolved: ResolvedTheme = systemDark ? "dark" : "light";
  const effectiveTheme: ResolvedTheme = preference === "system" ? systemResolved : preference;

  return (
    <ThemeContext.Provider
      value={{
        preference,
        systemResolved,
        effectiveTheme,
        setPreference,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}

