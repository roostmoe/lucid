import { createContext, useContext, useEffect, useState, type PropsWithChildren } from "react";
import type { User } from "../client";
import { useMutation, useQuery } from "@tanstack/react-query";
import { authLogoutMutation, authWhoamiOptions } from "../client/@tanstack/react-query.gen";

export type AuthContext = {
  loading: boolean;
  authenticated: boolean;
  user?: User;

  logout: () => Promise<void>;
};

const authContext = createContext<AuthContext>({
  loading: true,
  authenticated: false,
  logout: async () => {},
});

export const AuthProvider = ({ children }: PropsWithChildren) => {
  const { mutateAsync } = useMutation({
    ...authLogoutMutation({}),
    onSuccess: () => {
      window.location.reload();
    },
  });

  const [state, setState] = useState<AuthContext>({
    loading: true,
    authenticated: false,
    logout: async () => {
      await mutateAsync({});
    },
  });

  const { data: user, isLoading, error } = useQuery({
    ...authWhoamiOptions(),
    retry: false,
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
