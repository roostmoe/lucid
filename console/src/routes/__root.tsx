import type { AuthContext } from '@/lib/state/auth';
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';

const waitUntil = (condition: () => boolean, checkInterval=100) => {
  return new Promise(resolve => {
    let interval = setInterval(() => {
      if (!condition()) return;
      clearInterval(interval);
      resolve(null);
    }, checkInterval)
  });
}

export const Route = createRootRouteWithContext<{ auth: AuthContext }>()({
  loader: async ({ context }) => {
    await waitUntil(() => !context.auth.loading);
  },
  pendingComponent: () => {
    return (
      <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
        <span className="text-sm text-muted-foreground">Loading...</span>
      </div>
    )
  },
  component: RootComponent,
})

function RootComponent() {
  return (
    <>
      <Outlet />
      <TanStackRouterDevtools position="top-right" />
    </>
  )
}
