import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from "@tanstack/react-table";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "./ui/empty";
import { IconExclamationMark, IconFolderQuestion } from "@tabler/icons-react";
import type { UseQueryResult } from "@tanstack/react-query";
import type { AxiosError } from "axios";
import { Skeleton } from "./ui/skeleton";

export type DataTableProps<TData, TValue, TQueryResultData> = {
  columns: ColumnDef<TData, TValue>[];
  query: UseQueryResult<TQueryResultData, AxiosError<Error, any>>;
  queryResultDataToData?: (queryResultData?: TQueryResultData) => TData[];
};

export const DataTable = <TData, TValue, TQueryResultData>({
  columns,
  query: { data, error, isLoading, isFetching },
  queryResultDataToData = (queryResultData?: TQueryResultData) => queryResultData as unknown as TData[],
}: DataTableProps<TData, TValue, TQueryResultData>) => {
  const table = useReactTable({
    data: queryResultDataToData(data),
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="overflow-hidden rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id}>
                  {
                    header.isPlaceholder
                      ? null
                      : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )
                  }
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>

        <TableBody>
          {
            (isLoading || isFetching)
            ? (
              Array.from(Array(10), (_, i) => (
                <TableRow key={i}>
                  {Array.from(Array(columns.length), (_, ii) => (
                    <TableCell key={ii}>
                      <Skeleton className="h-4 w-16" />
                    </TableCell>
                  ))}
                </TableRow>
              ))
            )
            : (
              error
                ? <TableRow>
                    <TableCell colSpan={columns.length} className="text-center">
                      <Empty>
                        <EmptyHeader>
                          <EmptyMedia variant="icon">
                            <IconExclamationMark />
                          </EmptyMedia>
                          <EmptyTitle>We couldn't fetch that.</EmptyTitle>
                          <EmptyDescription>{error.response?.data.message} ({error.response?.data.code})</EmptyDescription>
                        </EmptyHeader>
                      </Empty>
                    </TableCell>
                  </TableRow>

                : table.getRowModel().rows?.length ? (
                    table.getRowModel().rows.map((row) => (
                      <TableRow
                        key={row.id}
                        data-state={row.getIsSelected() && "selected"}
                      >
                        {row.getVisibleCells().map((cell) => (
                          <TableCell key={cell.id}>
                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                          </TableCell>
                        ))}
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell colSpan={columns.length} className="text-center">
                        <Empty>
                          <EmptyHeader>
                            <EmptyMedia variant="icon">
                              <IconFolderQuestion />
                            </EmptyMedia>
                            <EmptyTitle>No data</EmptyTitle>
                            <EmptyDescription>We couldn&apos;t find any data to display.</EmptyDescription>
                          </EmptyHeader>
                        </Empty>
                      </TableCell>
                    </TableRow>
                  )
            )
          }
        </TableBody>
      </Table>
    </div>
  );
};
