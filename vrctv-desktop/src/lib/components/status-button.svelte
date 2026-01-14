<script lang="ts">
    const { status_good, children, href } = $props();
</script>

{#if status_good}
    <div class="p-8 rounded bg-green-800 text-center flex-1 text-white text-xl">
        {@render children()}
    </div>
{:else}
    <a
        class="p-8 rounded bg-red-600 dark:bg-red-800 dark:hover:bg-red-700 hover:bg-red-700 text-center flex-1 text-white text-xl"
        {href}
        target="_blank"
    >
        {@render children()}
    </a>
{/if}
