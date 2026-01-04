<script lang="ts">
    import { AndTrigger, type AndTriggerParams } from "$lib/triggers/and";
    import { triggers } from "$lib/task-parts";
    import type { TriggerInstance } from "$lib/triggers/types";
    import TriggerEditor from "../trigger-editor.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Trash } from "@lucide/svelte";
    import { OrTrigger, type OrTriggerParams } from "$lib/triggers/or";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    $effect(() => {
        if (!(trigger instanceof AndTrigger || trigger instanceof OrTrigger)) {
            trigger = new AndTrigger({});
        }
    });

    let params: AndTriggerParams | OrTriggerParams = $derived(trigger.params);
</script>

<div class="grid">
    {#each trigger.params.subtriggers as subtrigger, i}
        <div class="flex flex-row items-center gap-1 mt-2">
            <button
                class="m-1 bg-accent rounded hover:bg-accent/80 h-full p-1"
                onclick={() => {
                    params.subtriggers.splice(i, 1);
                    trigger = trigger;
                }}
            >
                <Trash class="text-red-500" />
            </button>
            <div class="flex flex-col h-full items-center w-2">
                <div
                    class="border border-l border-muted-foreground/50 h-full min-h-4"
                ></div>
            </div>
            <div class="pb-4 pt-2 flex flex-row">
                <TriggerEditor
                    bind:trigger={
                        () => trigger.params.subtriggers[i],
                        (s) => {
                            params.subtriggers[i] = s;
                            trigger.params = params;
                            trigger = trigger;
                        }
                    }
                />
            </div>
        </div>
    {/each}

    <Select.Root
        bind:value={
            () => "",
            (v) => {
                const ctor = triggers[v]?.trigger;

                if (ctor)
                    params.subtriggers.push(
                        new ctor({}) as TriggerInstance<any>,
                    );
                trigger.params = params;
                trigger = trigger;
            }
        }
        type="single"
    >
        <Select.Trigger class="mt-4 w-full max-w-lg text-muted-foreground"
            >Add New Trigger</Select.Trigger
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
</div>
