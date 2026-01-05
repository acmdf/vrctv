<script lang="ts">
    import { Copy, Trash } from "@lucide/svelte";
import Button from "../ui/button/button.svelte";

    const {
        ondelete,
        onduplicate,
    }: {
        ondelete?: () => void;
        onduplicate?: () => void;
    } = $props();
</script>

<div class="flex flex-row w-full justify-end gap-2 my-2">
    {#if onduplicate}
        <Button
            size="icon"
            variant="ghost"
            onclick={onduplicate}
        >
            <Copy />
        </Button>
    {/if}
    {#if ondelete}
        <Button
            size="icon"
            variant="destructive"
            onclick={ondelete}
        >
            <Trash />
        </Button>
    {/if}
</div>