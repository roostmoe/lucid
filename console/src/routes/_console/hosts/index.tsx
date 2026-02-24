import { AppSiteHeader } from '@/components/sidebar'
import { listHostsOptions } from '@/lib/client/@tanstack/react-query.gen'
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/hosts/')({
  component: RouteComponent,
})

function RouteComponent() {
  const { isLoading, data: hosts, error } = useQuery({ ...listHostsOptions() });

  return (
    <>
      <AppSiteHeader title="Hosts" />
      <div className="p-4 md:p-6">
        {(hosts ?? { items: [] }).items.map((host) => (
          <span>{host.hostname}</span>
        ))}
      </div>
    </>
  );
}
