import { Button } from "@/components/ui/button";
import { Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Field, FieldDescription, FieldGroup } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { createActivationKeyMutation, listActivationKeysQueryKey } from "@/lib/client/@tanstack/react-query.gen";
import type { ComponentRenderFn, DialogTriggerState, HTMLProps } from "@base-ui/react";
import { IconPlus } from "@tabler/icons-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState, type ChangeEvent, type SubmitEvent } from "react";

export type CreateActivationKeyModalProps = {
  button?: React.ReactElement<unknown, string | React.JSXElementConstructor<any>> | ComponentRenderFn<HTMLProps<any>, DialogTriggerState>;
};

export const CreateActivationKeyModal = ({ button }: CreateActivationKeyModalProps) => {
  const [id, setId] = useState('');
  const [open, setOpen] = useState(false);
  const [description, setDescription] = useState('');
  const queryClient = useQueryClient();

  const { mutate, data, isPending, isSuccess, reset } = useMutation({
    ...createActivationKeyMutation(),
    onSuccess: () => {
      setId('');
      setDescription('');
      queryClient.invalidateQueries({ queryKey: listActivationKeysQueryKey() });
    }
  });

  useEffect(() => {
    if (!open) {
      setId('');
      setDescription('');
      reset();
    }
  }, [open]);

  const handleSubmit = (e: SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    console.log('Create activation key with ID:', id, 'and description:', description);
    mutate({ body: { key_id: id, description } });
  };

  const handleIdChange = (e: ChangeEvent<HTMLInputElement>) => {
    let newVal = e.currentTarget.value;
    newVal = newVal.replace(/\s/g, '-');
    newVal = newVal.toLowerCase();
    setId(newVal);
  };

  return (
    <Dialog open={open} onOpenChange={open => setOpen(open)}>
        <DialogTrigger render={button ?? (
          <Button type="button">
            <IconPlus />
            Create Key
          </Button>
        )} />
        <DialogContent className="sm:max-w-sm">
          <DialogHeader className="mb-2">
            <DialogTitle>Create Activation Key</DialogTitle>
          </DialogHeader>

          {
            !isSuccess
            ? (
              <form onSubmit={handleSubmit}>
                <FieldGroup>
                  <Field>
                    <Label htmlFor="key_id">Key ID</Label>
                    <Input id="key_id" name="key_id" placeholder="A unique ID for this activation key." value={id} onChange={handleIdChange} />
                    <FieldDescription>This field must be unique across all activation keys.</FieldDescription>
                  </Field>

                  <Field>
                    <Label htmlFor="description">Description</Label>
                    <Textarea id="description" name="description" placeholder="A description for this activation key" value={description} onChange={e => setDescription(e.currentTarget.value)} />
                  </Field>
                </FieldGroup>

                <DialogFooter className="mt-4">
                  <DialogClose render={<Button variant="outline">Cancel</Button>} />
                  <Button type="submit" disabled={isPending}>Create</Button>
                </DialogFooter>
              </form>
            ) : (
              <div className="flex flex-col items-center justify-center gap-2 py-4 w-full overflow-x-auto">
                <p className="text-center">Activation key <span className="font-mono font-bold">{data?.key.key_id}</span> created successfully!</p>
                <pre className="w-full overflow-x-auto rounded bg-muted p-4">
                  <code className="rounded bg-muted p-2 text-sm">
                    {data?.token}
                  </code>
                </pre>
                <p>Please copy the above token now. You won't be able to see it again!</p>
              </div>
            )
          }
        </DialogContent>
    </Dialog>
  );
};
