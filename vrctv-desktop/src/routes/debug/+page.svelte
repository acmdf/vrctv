<script lang="ts">
  import { commands } from "../../bindings";
  import { oscStateStore, clientStateStore } from "$lib/stores/global";
  import type { PageProps } from "./$types";
  import { debug, warn } from "@tauri-apps/plugin-log";
  import { sendNotif, serverConnection } from "$lib/websocket";
  import toast from "svelte-french-toast";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import {
    eventLogStore,
    serviceStateStore,
    taskStateStore,
  } from "$lib/stores/debug";
  import {
    customRewardsStore,
    defaultRewardStore,
    rewardHandler,
    rewardStore,
  } from "$lib/stores/rewards";
  import ServerSelectorDialogue from "$lib/components/server-selector-dialogue.svelte";

  const { data }: PageProps = $props();

  let serverSelectOpen = $state<boolean>(false);
  let setParam = $state<string>("");
  let setValue = $state<string>("");
  let avatarId = $state<string>("");

  const params = $derived(await commands.fetchAvatarOsc(avatarId));

  $effect(() => {
    if (params.status !== "ok") {
      toast.error(`Error fetching OSC params: ${params.error}`);
    }
  });

  const avatars = $derived(
    data.status === "ok"
      ? data.data.sort((a, b) => a.name.localeCompare(b.name))
      : []
  );

  function formatValue(value: any): string {
    if (typeof value === "object") {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  }

  $effect(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Debug Page"
        ),
      },
    });
  });

  warn(`Debug page data: ${JSON.stringify(data)}`);
</script>

<ServerSelectorDialogue bind:open={serverSelectOpen} />

<h1 class="text-3xl font-bold mb-4">Debug Information</h1>
<div class="flex w-full space-x-4 mb-4">
  <button
    class="p-8 rounded bg-grey-800 text-center flex-1 hover:bg-gray-700"
    onclick={() =>
      sendNotif(
        "Test Notification",
        "This is a test notification from the debug page."
      )}
  >
    Test notification
  </button>
  <button
    class="p-8 rounded bg-grey-800 text-center flex-1 hover:bg-gray-700"
    onclick={() => rewardStore.set(defaultRewardStore)}
  >
    Set default rewards
  </button>
  <button
    class="p-8 rounded bg-grey-800 text-center flex-1 hover:bg-gray-700"
    onclick={() => (serverSelectOpen = true)}
  >
    Open server selector
  </button>
</div>

<h2 class="text-2xl font-bold mb-2">Set OSC Parameter</h2>
<div class="mb-4 flex flex-wrap">
  <select
    bind:value={avatarId}
    class="p-2 bg-gray-800 text-white rounded w-1/3 mr-2 mb-2"
  >
    {#each avatars as avatar}
      <option value={avatar.id}>{avatar.name} ({avatar.id})</option>
    {/each}
  </select>
  {#if params.status === "ok"}
    <select
      bind:value={setParam}
      class="p-2 bg-gray-800 text-white rounded w-1/3 mr-2 mb-2"
    >
      {#each params.data as param}
        <option value={`/avatar/parameters/${param}`}>{param}</option>
      {/each}
    </select>
    <input
      type="text"
      placeholder="Parameter Value"
      bind:value={setValue}
      class="p-2 bg-gray-800 text-white rounded w-1/3 mr-2"
    />
    <button
      class="p-2 bg-gray-800 text-white rounded hover:bg-gray-700"
      onclick={async () => {
        toast.success(
          JSON.stringify(await commands.setOsc(setParam, setValue))
        );
      }}
    >
      Set Parameter
    </button>
  {:else}
    <p class="text-red-500">Error: {params.error}</p>
  {/if}
</div>

<h2 class="text-2xl font-bold mb-2">Test Warudo OSC Parameter</h2>
<Input
  type="text"
  placeholder="Parameter Name"
  bind:value={setParam}
  class="p-2 bg-gray-800 text-white rounded w-1/3 mr-2 mb-2"
/>
<Input
  type="text"
  placeholder="Parameter Value"
  bind:value={setValue}
  class="p-2 bg-gray-800 text-white rounded w-1/3 mr-2 mb-2"
/>
<Button
  class="p-2 bg-gray-800 text-white rounded hover:bg-gray-700 mb-4"
  onclick={async () => {
    toast.success(
      JSON.stringify(await commands.setWarudoOsc(setParam, setValue))
    );
  }}
>
  Set Warudo Parameter
</Button>

{#snippet debugTable(title: string, data: Record<string, any>)}
  <h2 class="text-2xl font-bold mb-2">{title}</h2>
  <div class="mb-4">
    {#each Object.entries(data) as [key, value]}
      <div class="flex p-2 items-center">
        <p class="text-md bg-gray-800 rounded p-2 mr-2">{key}</p>
        {formatValue(value)}
      </div>
    {/each}
  </div>
{/snippet}

{@render debugTable("Client Info", $clientStateStore)}
{@render debugTable("Task Info", $taskStateStore)}
{@render debugTable("Reward Store", $rewardStore)}
{@render debugTable("Custom Rewards", $customRewardsStore)}
{@render debugTable("Event Log", $eventLogStore)}
{@render debugTable("OSC State", $oscStateStore)}
{@render debugTable("Active Rewards", $rewardHandler.activeRewards)}
{@render debugTable("Reward Queue", $rewardHandler.rewardQueue)}
{@render debugTable("Global KV", $rewardHandler.globalKV)}

<h2 class="text-2xl font-bold mt-4 mb-2">Service Status</h2>
{#each Object.entries($serviceStateStore) as [service, status]}
  <div class="flex p-2 items-center">
    <p class="mr-2">
      {status === "Started" ? "🟢" : status === "Stopped" ? "🔴" : "⚠️"}
    </p>
    <p class="text-md bg-gray-800 rounded p-2 mr-2">{service}</p>
    {typeof status === "object" ? "Error" : status}
    {#if typeof status === "object"}
      <div class="ml-4 p-2 bg-gray-700 rounded">
        <pre>{status.Error}</pre>
      </div>
    {/if}
  </div>
{/each}

{#if data.status == "ok"}
  <h2 class="text-2xl font-bold mb-2">Fetched Avatars</h2>
  <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
    {#each avatars as avatar}
      <button
        class="flex flex-col items-center p-2 bg-gray-800 rounded cursor-pointer hover:bg-gray-700"
        onclick={() => commands.changeAvatar(avatar.id)}
      >
        <p class="text-sm">{avatar.name}</p>
        <p class="text-xs text-gray-400">{avatar.id}</p>
      </button>
    {/each}
  </div>
{:else}
  <h2 class="text-2xl font-bold mb-2">No avatars fetched</h2>
  <p class="text-red-500">Error: {data.error}</p>
{/if}
