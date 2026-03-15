import createStore from "@/lib/makeStore";

function initialState(): string {
  const queries = localStorage.getItem("sql");
  if (queries !== null) {
    return queries;
  }

  return "select 1 + 1;";
}

export const {
  StoreProvider: SqlProvider,
  useDispatch: useSqlDispatch,
  useStore: useSql,
} = createStore((_, action: { type: "SET_SQL"; data: string }) => {
  switch (action.type) {
    case "SET_SQL":
      localStorage.setItem("sql", action.data);
      return action.data;
  }
}, initialState());
