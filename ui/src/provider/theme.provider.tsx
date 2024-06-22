import createStore from "@/lib/makeStore";

export type Theme = "dark" | "light";

function initialState(): Theme {
  if (localStorage.getItem("theme") !== null) {
    return localStorage.getItem("theme") === "dark" ? "dark" : "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export const {
  StoreProvider: ThemeProvider,
  useDispatch: setTheme,
  useStore: useTheme,
} = createStore((_, next: Theme) => {
  localStorage.setItem("theme", next);
  return next;
}, initialState());
