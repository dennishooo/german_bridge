import { writable } from "svelte/store";
import { browser } from "$app/environment";

type Theme = "light" | "dark";

function createThemeStore() {
  const getInitialTheme = (): Theme => {
    if (!browser) return "light";

    const stored = localStorage.getItem("theme") as Theme | null;
    if (stored) return stored;

    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  };

  const { subscribe, set } = writable<Theme>(getInitialTheme());

  return {
    subscribe,
    toggle: () => {
      if (!browser) return;

      const current = document.documentElement.classList.contains("dark")
        ? "dark"
        : "light";
      const next = current === "light" ? "dark" : "light";

      document.documentElement.classList.remove(current);
      document.documentElement.classList.add(next);
      localStorage.setItem("theme", next);
      set(next);
    },
    init: () => {
      if (!browser) return;

      const theme = getInitialTheme();
      document.documentElement.classList.add(theme);
      set(theme);
    },
  };
}

export const theme = createThemeStore();
