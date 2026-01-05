<script lang="ts">
    import { type WithElementRef } from "$lib/utils.js";
    import type { HTMLAttributes } from "svelte/elements";
    import {
        QueueCollapseState,
        setCollapseContext,
    } from "./context.svelte.js";
    import { Minus, Plus } from "@lucide/svelte";
    import Button from "../ui/button/button.svelte";

    let {
        open = $bindable(false),
        onOpenChange = () => {},
        children,
    }: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
        open?: boolean;
        onOpenChange?: (open: boolean) => void;
    } = $props();

    const collapsed = setCollapseContext(
        new QueueCollapseState({
            open: () => open,
            setOpen: (value: boolean) => {
                open = value;
                onOpenChange(value);
            },
        }),
    );
</script>

<div class="flex flex-row items-center gap-1 my-2">
    <div class="flex flex-col h-full items-center">
        {#if !open}
            <Button
                size="icon"
                variant="ghost"
                onclick={() => {
                    collapsed.setOpen(true);
                }}
            >
                <Plus />
            </Button>
        {:else}
            <Button
                size="icon"
                variant="ghost"
                onclick={() => {
                    collapsed.setOpen(false);
                }}
            >
                <Minus />
            </Button>
        {/if}
        <div
            class="border border-l border-muted-foreground/50 h-full min-h-4"
        ></div>
    </div>
    <div class="flex-1">
        {@render children?.()}
    </div>
</div>
