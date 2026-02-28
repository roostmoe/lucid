import { RouterProvider as TsrRouterProvider, createRouter } from "@tanstack/react-router";
import { routeTree } from './routeTree.gen';
import { useAuth } from "./lib/state/auth";

export const RouterProvider = () => {
  const auth = useAuth();
  const router = createRouter({
    routeTree,
    context: {
      auth,
    }
  });
  return (
    <TsrRouterProvider router={router} />
  );
};
