import { DataTable } from '@/components/data-table';
import { AppSiteHeader } from '@/components/sidebar'
import { keysTableColumns } from '@/components/views/activation-keys/keys-table';
import { listActivationKeysOptions } from '@/lib/client/@tanstack/react-query.gen'
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/activation-keys/')({
  component: RouteComponent,
})

function RouteComponent() {
  const query = useQuery({ ...listActivationKeysOptions() });

  return (
    <>
      <AppSiteHeader title="Activation Keys" />
      <div className="flex flex-1 flex-col">
        <div className="@container/main flex flex-1 flex-col gap-2">
          <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
            <div className="px-4 lg:px-6">
              <DataTable
                columns={keysTableColumns}
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
