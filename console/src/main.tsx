import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { RouterProvider } from "./router.tsx"
import "./index.css"
import { ReactQueryDevtools } from "@tanstack/react-query-devtools"
import { QueryProvider } from "./lib/query/index.tsx"
import { AuthProvider } from "./lib/state/auth.tsx"

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryProvider>
      <AuthProvider>
        <RouterProvider />
      </AuthProvider>
      <ReactQueryDevtools />
    </QueryProvider>
  </StrictMode>
)
