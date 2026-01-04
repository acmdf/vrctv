<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import { triggers } from "$lib/task-parts";
    import { TriggerInstance } from "$lib/triggers/types";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    const EditorComponent = $derived(triggers[trigger.trigger.id]?.editor);
</script>

<div class="grid items-center gap-1.5">
    <Label>Type</Label>
    <Select.Root
        bind:value={
            () => trigger.trigger.id,
            (v) => {
                const ctor = triggers[v].trigger;

                if (ctor) trigger = new ctor({}) ?? trigger;
            }
        }
        type="single"
    >
        <Select.Trigger class="mb-4 w-full max-w-lg"
            >{trigger.trigger.title}</Select.Trigger
        >
        <Select.Content align="start">
            {#each Object.entries(triggers) as [id, triggerTypeOption]}
                <Select.Item
                    value={id}
                    class="hover:bg-accent hover:text-accent-foreground"
                >
                    <div class="flex flex-col">
                        <span>{triggerTypeOption.trigger.title}</span>
                        <span class="text-sm text-muted-foreground">
                            {triggerTypeOption.trigger.description}
                        </span>
                    </div>
                </Select.Item>
            {/each}
        </Select.Content>
    </Select.Root>

    {#if EditorComponent}
        <EditorComponent bind:trigger />
    {/if}
</div>
