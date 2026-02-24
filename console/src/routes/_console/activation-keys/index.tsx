import { DataTable } from '@/components/data-table';
import { AppSiteHeader } from '@/components/sidebar'
import { Separator } from '@/components/ui/separator';
import { CreateActivationKeyModal } from '@/components/views/activation-keys/create-modal';
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
      <AppSiteHeader title="Activation Keys">
        <CreateActivationKeyModal />
      </AppSiteHeader>
      <div className="flex flex-1 flex-col">
        <div className="@container/main flex flex-1 flex-col gap-2">
          <div className="flex flex-col py-4 md:py-6">
            <div className="flex flex-col gap-2 px-4 lg:px-6 mb-4 md:mb-6">
              <h1 className="text-2xl font-semibold tracking-tight">Activation Keys</h1>
              <p className="text-muted-foreground">
                Activation keys are used to register hosts to your Lucid
                instance. They can be created with an optional description and
                are identified by a unique key ID.
              </p>
            </div>

            <Separator />

            <DataTable
              columns={keysTableColumns}
              query={query}
              queryResultDataToData={q => q ? q.items ?? [] : []}
              embedded
              empty={{
                title: 'No activation keys found',
                description: 'Create activation keys to register hosts to your Lucid instance.',
                actions: (
                  <CreateActivationKeyModal button={undefined} />
                ),
                learnMore: 'https://lucid.roost.moe/docs/activation-keys'
              }}
            />
          </div>
        </div>
      </div>
    </>
  );
}
