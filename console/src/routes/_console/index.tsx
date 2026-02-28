import { ComponentExample } from '@/components/component-example';
import { AppSiteHeader } from '@/components/sidebar';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <>
      <AppSiteHeader title="Dashboard" />
      <ComponentExample />
    </>
  );
}
