import { ComponentExample } from '@/components/component-example';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  component: RouteComponent,
})

function RouteComponent() {
  return <ComponentExample />;
}
