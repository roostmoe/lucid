import { DataTable } from '@/components/data-table';
import { AppSiteHeader } from '@/components/sidebar'
import { Separator } from '@/components/ui/separator';
import { casTableColumns } from '@/components/views/certificate-authorities/cas-table';
import { CreateCaModal } from '@/components/views/certificate-authorities/create-modal';
import { listCasOptions } from '@/lib/client/@tanstack/react-query.gen';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_console/admin/certificate-authorities/')({
  component: RouteComponent,
})

function RouteComponent() {
  const query = useQuery({ ...listCasOptions() });

  return (
    <>
      <AppSiteHeader title="Certificate Authorities">
        <CreateCaModal />
      </AppSiteHeader>
      <div className="flex flex-1 flex-col">
        <div className="@container/main flex flex-1 flex-col gap-2">
          <div className="flex flex-col py-4 md:py-6">
            <div className="flex flex-col gap-2 px-4 lg:px-6 mb-4 md:mb-6">
              <h1 className="text-2xl font-semibold tracking-tight">Certificate Authorities</h1>
              <p className="text-muted-foreground">
                Certificate Authorities are used for several purposes within
                Lucid. Primarily, they are used to secure the connections
                between the server and host agents, but they can also be used to
                secure repositories as well as other tasks.
              </p>
            </div>

            <Separator />

            <DataTable
              columns={casTableColumns}
              query={query}
              queryResultDataToData={q => q ? q.items ?? [] : []}
              embedded
              empty={{
                title: 'No CAs found',
                description: 'Create CAs to register hosts to your Lucid instance.',
                learnMore: 'https://lucid.roost.moe/docs/admin/certificate-authorities',
                actions: (<CreateCaModal />),
              }}
            />
          </div>
        </div>
      </div>
    </>
  );
}
