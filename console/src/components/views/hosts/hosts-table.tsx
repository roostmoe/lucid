import type { Host } from "@/lib/client";
import { Link } from "@tanstack/react-router";
import type { ColumnDef } from "@tanstack/react-table";
import 'font-logos/assets/font-logos.css';

const iconMapper = {
  'rocky': 'rocky-linux',
} as const;

const isIconMapperKey = (id: string): id is keyof typeof iconMapper => {
  if (Object.keys(iconMapper).includes(id)) {
    return true;
  }
  return false;
}

const icon = (id: keyof typeof iconMapper | string) => {
  if (isIconMapperKey(id)) {
    return iconMapper[id];
  }
  return 'tux';
};

export const hostsTableColumns: ColumnDef<Host>[] = [
  {
    accessorKey: 'hostname',
    header: 'Hostname',
    cell: ({ row }) => {
      const hostname = row.getValue('hostname') as string;
      return (
        <Link to="/hosts/$hostId" params={{ hostId: row.original.id }}>
          <span className="text-primary hover:underline">
            {hostname}
          </span>
        </Link>
      );
    },
  },

  {
    id: 'operatingSystem',
    header: 'Operating System',
    accessorFn: (row) => `${row.os_name} ${row.os_version}`,
    cell: ({ row }) => {
      return (
        <div className="flex items-center gap-1">
          <i className={`fl-${icon(row.original.os_id)}`} />
          <span>{row.getValue('operatingSystem')}</span>
        </div>
      )
    },
  },

  {
    id: 'registeredAt',
    accessorFn: (row) => new Date(row.created_at).toLocaleString(),
    header: 'Registered At',
  },

  {
    id: 'lastSeenAt',
    accessorFn: (row) => new Date(row.last_seen_at).toLocaleString(),
    header: 'Last Seen',
  },
];
