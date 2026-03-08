import { Button } from "@/components/ui/button";
import type { Ca } from "@/lib/client";
import { deleteCaMutation, listCasQueryKey } from "@/lib/client/@tanstack/react-query.gen";
import { IconTrash } from "@tabler/icons-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { ColumnDef } from "@tanstack/react-table";

export const casTableColumns: ColumnDef<Ca>[] = [
  {
    accessorKey: 'id',
    header: 'CA ID',
    cell: ({ row }) => {
      const keyId = row.getValue('id') as string;

      return (
        <pre className="py-1 px-2 inline bg-muted rounded-sm font-mono text-sm">
          {keyId}
        </pre>
      );
    },
  },

  {
    accessorKey: 'fingerprint',
    header: 'Fingerprint',
  },

  {
    accessorKey: 'created_at',
    header: 'Created At',
  },

  {
    id: 'actions',
    header: 'Actions',
    cell: ({ row }) => {
      const id = row.original.id;
      const queryClient = useQueryClient();

      const { mutate } = useMutation({
        ...deleteCaMutation(),
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: listCasQueryKey() });
        },
      });

      return (
        <Button size="icon-sm" variant="destructive" onClick={() => mutate({ path: { id } })}>
          <IconTrash />
        </Button>
      );
    },
  }
];
