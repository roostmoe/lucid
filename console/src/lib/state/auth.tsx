import { createContext, useContext, useEffect, useState, type PropsWithChildren } from "react";
import type { User } from "../client";
import { useQuery } from "@tanstack/react-query";
import { authWhoamiOptions } from "../client/@tanstack/react-query.gen";

export type AuthContext = {
  loading: boolean;
  authenticated: boolean;
  user?: User;
};

const authContext = createContext<AuthContext>({
  loading: true,
  authenticated: false,
});

export const AuthProvider = ({ children }: PropsWithChildren) => {
  const [state, setState] = useState<AuthContext>({
    loading: true,
    authenticated: false,
  });

  const { data: user, isLoading, error } = useQuery({
    ...authWhoamiOptions(),
  });

  useEffect(() => {
    if (!isLoading) {
      setState({
        ...state,
        loading: false,
        authenticated: !error,
        user,
      });
    }
  }, [user, isLoading, error]);

  return (
    <authContext.Provider value={state}>{children}</authContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(authContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
