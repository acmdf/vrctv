<script lang="ts">
    import { getCollapseContext } from "./context.svelte";

    const { children } = $props();

    const collapsed = getCollapseContext();
</script>

{#if collapsed.open}
    {@render children?.()}
{/if}
