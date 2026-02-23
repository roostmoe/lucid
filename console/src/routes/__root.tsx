import { Outlet, createRootRoute } from '@tanstack/react-router'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { QueryProvider } from '@/lib/query'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';

export const Route = createRootRoute({
  component: RootComponent,
})

function RootComponent() {
  return (
    <>
      <QueryProvider>
        <Outlet />
        <ReactQueryDevtools />
      </QueryProvider>
      <TanStackRouterDevtools />
    </>
  )
}
