<script>
  import { backendUrl, clientStateStore } from "$lib/stores/global";
  import StatusButton from "$lib/components/statusButton.svelte";
</script>

<div class="flex w-full space-x-4 mb-4">
  {#if $clientStateStore.connected}
    <StatusButton
      status_good={$clientStateStore.twitch_name}
      href={`${$backendUrl}twitch/auth/${$clientStateStore.id}`}
    >
      {#if $clientStateStore.twitch_name}
        Connected to Twitch as {$clientStateStore.twitch_name}
      {:else}
        Connect to Twitch
      {/if}
    </StatusButton>
    <StatusButton
      status_good={$clientStateStore.has_streamlabs}
      href={`${$backendUrl}streamlabs/auth/${$clientStateStore.id}`}
    >
      {#if $clientStateStore.has_streamlabs}
        Connected to Streamlabs as {$clientStateStore.streamlabs_name}
      {:else}
        Connect to Streamlabs
      {/if}
    </StatusButton>
  {:else}
    <div class="p-8 rounded dark:bg-gray-800 bg-gray-300 text-center flex-1">
      Not connected to backend
    </div>
  {/if}
</div>
