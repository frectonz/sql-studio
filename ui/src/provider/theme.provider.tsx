import createStore from "@/lib/makeStore";

export type Theme = "dark" | "light";

export const {
  StoreProvider: ThemeProvider,
  useDispatch: setTheme,
  useStore: useTheme,
} = createStore(
  (_, next: Theme) => {
    localStorage.setItem("theme", next);
    return next;
  },
  (localStorage.getItem("theme") === "dark" ? "dark" : "light") as Theme
);
