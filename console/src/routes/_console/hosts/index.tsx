import { DataTable } from '@/components/data-table';
import { AppSiteHeader } from '@/components/sidebar'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { hostsTableColumns } from '@/components/views/hosts/hosts-table';
import { listHostsOptions } from '@/lib/client/@tanstack/react-query.gen'
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/hosts/')({
  component: RouteComponent,
})

function RouteComponent() {
  const query = useQuery({ ...listHostsOptions() });

  return (
    <>
      <AppSiteHeader title="Hosts" />
      <div className="flex flex-1 flex-col">
        <div className="@container/main flex flex-1 flex-col gap-2">
          <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
            <div className="*:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card dark:*:data-[slot=card]:bg-card grid grid-cols-1 gap-4 px-4 *:data-[slot=card]:bg-gradient-to-t *:data-[slot=card]:shadow-xs lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4">
              <Card className="@container/card">
                <CardHeader>
                  <CardDescription>Total hosts</CardDescription>
                  <CardTitle>{(query.data ? (query.data.items ?? []).length : 0)}</CardTitle>
                </CardHeader>
              </Card>
            </div>

            <div className="px-4 lg:px-6">
            <DataTable
              columns={hostsTableColumns}
              query={query}
              queryResultDataToData={q => q ? q.items ?? [] : []}
            />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
