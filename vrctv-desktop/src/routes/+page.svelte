<script>
  import { clientStateStore } from "$lib/stores";
  import { BACKEND_URL } from "../consts";
</script>

<div class="flex w-full space-x-4 mb-4">
  {#if $clientStateStore.connected}
    {#if $clientStateStore.has_twitch}
      <div class="p-8 rounded bg-green-800 text-center flex-1">
        Connected to Twitch as {$clientStateStore.twitch_name}
      </div>
    {:else}
      <a
        class="p-8 rounded bg-red-800 text-center flex-1"
        href={`${BACKEND_URL}twitch/auth/${$clientStateStore.id}`}
        target="_blank">Connect to Twitch</a
      >
    {/if}
    {#if $clientStateStore.has_streamlabs}
      <div class="p-8 rounded bg-green-800 text-center flex-1">
        Connected to Streamlabs as {$clientStateStore.streamlabs_name}
      </div>
    {:else}
      <a
        class="p-8 rounded bg-red-800 text-center flex-1"
        href={`${BACKEND_URL}streamlabs/auth/${$clientStateStore.id}`}
        target="_blank">Connect to Streamlabs</a
      >
    {/if}
  {:else}
    <div class="p-8 rounded bg-gray-800 text-center flex-1">
      Not connected to backend
    </div>
  {/if}
</div>
