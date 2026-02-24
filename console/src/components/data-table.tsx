import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from "@tanstack/react-table";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "./ui/empty";
import { IconArrowUpRight, IconExclamationMark, IconFolderQuestion, type Icon } from "@tabler/icons-react";
import type { UseQueryResult } from "@tanstack/react-query";
import type { AxiosError } from "axios";
import { Skeleton } from "./ui/skeleton";
import { Button } from "./ui/button";
import { Link } from "@tanstack/react-router";

export type DataTableProps<TData, TValue, TQueryResultData> = {
  columns: ColumnDef<TData, TValue>[];
  query: UseQueryResult<TQueryResultData, AxiosError<Error, any>>;
  queryResultDataToData?: (queryResultData?: TQueryResultData) => TData[];
  empty?: {
    title?: string;
    description?: string;
    learnMore?: string;
    actions?: {
      title: string,
      icon?: Icon,
      link: string,
      variant: 'default' | 'outline' | 'link',
    }[];
  };
};

export const DataTable = <TData, TValue, TQueryResultData>({
  columns,
  query: { data, error, isLoading, isFetching },
  queryResultDataToData = (queryResultData?: TQueryResultData) => queryResultData as unknown as TData[],
  empty = {
    title: 'No data',
    description: 'We couldn\'t find any data to display.',
    actions: [],
  },
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
                            <EmptyTitle>{empty.title}</EmptyTitle>
                            <EmptyDescription>{empty.description}</EmptyDescription>
                          </EmptyHeader>
                          <EmptyContent className="flex-row justify-center gap-2">
                            {
                              empty.actions?.map((item, i) => (
                                <Button key={i} variant={item.variant} render={(
                                  <Link to={item.link} target={item.link.startsWith('http') ? '_blank' : undefined}>
                                    {item.icon && <item.icon />}
                                    {item.title}
                                    {item.link.startsWith('http') && <IconArrowUpRight />}
                                  </Link>
                                )} />
                              ))
                            }
                          </EmptyContent>
                          {empty.learnMore && (
                            <Button variant="link" render={(
                              <Link to={empty.learnMore} target="_blank">
                                Learn more
                                <IconArrowUpRight />
                              </Link>
                            )} />
                          )}
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
