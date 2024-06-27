import createStore from "@/lib/makeStore";

export type SavedQuery = {
  name: string;
  query: string;
};
export type SavedQueries = SavedQuery[];
export type Action =
  | {
      type: "SAVE_QUERY";
      data: SavedQuery;
    }
  | {
      type: "UPDATE_QUERY";
      index: number;
      data: string;
    }
  | {
      type: "DELETE_QUERY";
      index: number;
    };

function initialState(): SavedQueries {
  const queries = localStorage.getItem("queries");
  if (queries !== null) {
    return JSON.parse(queries);
  }

  return [];
}

export const {
  StoreProvider: QueriesProvider,
  useDispatch: useQueriesDispatch,
  useStore: useQueries,
} = createStore((queries, action: Action) => {
  switch (action.type) {
    case "SAVE_QUERY":
      const saved = [...queries, action.data];
      localStorage.setItem("queries", JSON.stringify(saved));
      return saved;

    case "UPDATE_QUERY":
      const updated = queries.map((q, i) =>
        i === action.index ? { ...q, query: action.data } : q,
      );
      localStorage.setItem("queries", JSON.stringify(updated));
      return updated;

    case "DELETE_QUERY":
      const deleted = queries.filter((_, i) => i !== action.index);
      localStorage.setItem("queries", JSON.stringify(deleted));
      return deleted;
  }
}, initialState());
