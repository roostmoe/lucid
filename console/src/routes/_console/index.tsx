import { ComponentExample } from '@/components/component-example';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <>
      <ComponentExample />
    </>
  );
}
