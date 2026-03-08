import { Button } from "@/components/ui/button";
import { Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { generateCaMutation, listCasQueryKey } from "@/lib/client/@tanstack/react-query.gen";
import type { ComponentRenderFn, DialogTriggerState, HTMLProps } from "@base-ui/react";
import { IconPlus } from "@tabler/icons-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState, type SubmitEvent } from "react";

export type CreateActivationKeyModalProps = {
  button?: React.ReactElement<unknown, string | React.JSXElementConstructor<any>> | ComponentRenderFn<HTMLProps<any>, DialogTriggerState>;
};

export const CreateCaModal = ({ button }: CreateActivationKeyModalProps) => {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const { mutate, isPending, reset } = useMutation({
    ...generateCaMutation(),
    onSuccess: () => {
      setOpen(false);
      queryClient.invalidateQueries({ queryKey: listCasQueryKey() });
    }
  });

  useEffect(() => {
    if (!open) {
      reset();
    }
  }, [open]);

  const handleSubmit = (e: SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    mutate({});
  };

  return (
    <Dialog open={open} onOpenChange={open => setOpen(open)}>
        <DialogTrigger render={button ?? (
          <Button type="button">
            <IconPlus />
            Create CA
          </Button>
        )} />
        <DialogContent className="sm:max-w-sm">
          <DialogHeader className="mb-2">
            <DialogTitle>Create a new CA</DialogTitle>
          </DialogHeader>

          <form onSubmit={handleSubmit}>
            <DialogFooter className="mt-4">
              <DialogClose render={<Button variant="outline">Cancel</Button>} />
              <Button type="submit" disabled={isPending}>Create</Button>
            </DialogFooter>
          </form>
        </DialogContent>
    </Dialog>
  );
};
