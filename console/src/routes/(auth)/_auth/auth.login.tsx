import { LoginForm } from '@/components/views/auth/login';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/(auth)/_auth/auth/login')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <div className="w-full max-w-sm">
      <LoginForm />
    </div>
  );
}
