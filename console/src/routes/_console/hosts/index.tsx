import { DataTable } from '@/components/data-table';
import { AppSiteHeader } from '@/components/sidebar'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
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
          <div className="flex flex-col py-4 md:py-6">
            <div className="flex flex-col gap-2 px-4 lg:px-6 mb-4 md:mb-6">
              <h1 className="text-2xl font-semibold tracking-tight">Hosts</h1>
              <p className="text-muted-foreground">
                Hosts are the machines that are registered to your Lucid
                instance. They can be registered using activation keys and are
                tracked for inventory purposes here.
              </p>
            </div>

            <div className="*:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card dark:*:data-[slot=card]:bg-card grid grid-cols-1 gap-4 px-4 *:data-[slot=card]:bg-gradient-to-t *:data-[slot=card]:shadow-xs lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4 mb-4 md:mb-6">
              <Card className="@container/card">
                <CardHeader>
                  <CardDescription>Total hosts</CardDescription>
                  <CardTitle>{(query.data ? (query.data.items ?? []).length : 0)}</CardTitle>
                </CardHeader>
              </Card>
            </div>

            <Separator />

            <DataTable
              embedded
              columns={hostsTableColumns}
              query={query}
              queryResultDataToData={q => q ? q.items ?? [] : []}
            />
          </div>
        </div>
      </div>
    </>
  );
}
