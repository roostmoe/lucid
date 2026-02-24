import { Button } from "@/components/ui/button";
import type { ActivationKey } from "@/lib/client";
import { deleteActivationKeyMutation, listActivationKeysQueryKey } from "@/lib/client/@tanstack/react-query.gen";
import { IconTrash } from "@tabler/icons-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { ColumnDef } from "@tanstack/react-table";

export const keysTableColumns: ColumnDef<ActivationKey>[] = [
  {
    accessorKey: 'key_id',
    header: 'Key ID',
    cell: ({ row }) => {
      const keyId = row.getValue('key_id') as string;

      return (
        <pre className="py-1 px-2 inline bg-muted rounded-sm font-mono text-sm">
          {keyId}
        </pre>
      );
    },
  },

  {
    accessorKey: 'description',
    header: 'Description',
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
        ...deleteActivationKeyMutation(),
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: listActivationKeysQueryKey() });
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
