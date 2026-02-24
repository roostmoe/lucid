import type { ActivationKey } from "@/lib/client";
import type { ColumnDef } from "@tanstack/react-table";

export const keysTableColumns: ColumnDef<ActivationKey>[] = [
  {
    accessorKey: 'key_id',
    header: 'Key ID',
  },

  {
    accessorKey: 'description',
    header: 'Description',
  },

  {
    accessorKey: 'created_at',
    header: 'Created At',
  },
];
