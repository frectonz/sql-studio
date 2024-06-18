import { Dispatch, PropsWithChildren, createContext, useContext, useReducer } from "react";

/**
 * Creates a store with the specified reducer and initial state.
 *
 * @template State - The type of the state.
 * @template Action - The type of the action.
 * @param {function(state: State, action: Action): State} reducer - The reducer function that updates the state based on the action.
 * @param {State} initialState - The initial state of the store.
 * @returns {{ StoreProvider: React.ComponentType<PropsWithChildren>, useDispatch: function(): Dispatch<Action>, useStore: function(): State }} - An object containing the StoreProvider component, useDispatch hook, and useStore hook.
 */
export default function createStore<State, Action>(
  reducer: (state: State, action: Action) => State,
  initialState: State
) {
  const StoreContext = createContext(initialState);
  const DispatchContext = createContext<Dispatch<Action>>(() => {});

  function StoreProvider({ children }: PropsWithChildren) {
    const [store, dispatch] = useReducer(reducer, initialState);

    return (
      <StoreContext.Provider value={store}>
        <DispatchContext.Provider value={dispatch}>{children}</DispatchContext.Provider>
      </StoreContext.Provider>
    );
  }

  function useStore() {
    return useContext(StoreContext);
  }

  function useDispatch() {
    return useContext(DispatchContext);
  }
  return { StoreProvider, useDispatch, useStore };
}
