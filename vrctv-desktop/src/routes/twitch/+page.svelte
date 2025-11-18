<script>
  import { customRewardsStore } from "$lib/stores";
  import { serverConnection } from "$lib/websocket";
  import { info } from "@tauri-apps/plugin-log";
  import { onMount } from "svelte";

  let lastCustomRewardsState = $state($customRewardsStore);
  let currentCustomRewards = $state($customRewardsStore);
  let touched = $state(false);

  $effect(() => {
    if (
      JSON.stringify($customRewardsStore) !==
      JSON.stringify(lastCustomRewardsState)
    ) {
      info(`Custom rewards changed: ${JSON.stringify($customRewardsStore)}`);

      currentCustomRewards = $customRewardsStore;
      lastCustomRewardsState = $customRewardsStore;
      touched = false;
    } else if (
      !touched &&
      JSON.stringify(currentCustomRewards) !==
        JSON.stringify(lastCustomRewardsState)
    ) {
      touched = true;
    }
  });

  onMount(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Twitch Page",
        ),
      },
    });
  });
</script>

<h1 class="text-3xl font-bold mb-4">Twitch Custom Rewards</h1>
{#if touched}
  <div class="mb-4 p-4 bg-yellow-200 text-yellow-800 rounded">
    You have unsaved changes. Please save to apply them.
  </div>
{/if}

{#if Object.keys(currentCustomRewards).length === 0}
  <div class="p-4 bg-gray-800 rounded">No custom rewards found.</div>
{:else}
  <div class="space-y-4">
    {#each Object.entries(currentCustomRewards) as [rewardId, reward]}
      <div class="p-4 bg-gray-800 rounded">
        <input
          type="text"
          bind:value={reward.title}
          class="text-xl font-semibold mb-2 bg-gray-700 text-white p-1 rounded w-full"
        />
        <textarea
          bind:value={reward.prompt}
          class="mb-2 bg-gray-700 text-white p-1 rounded w-full"
        ></textarea>
        <p class="mb-2">
          Cost: <input
            type="number"
            bind:value={reward.cost}
            class="bg-gray-700 text-white p-1 rounded"
          /> points
        </p>
        <p class="mb-2">Status: {reward.is_enabled ? "Enabled" : "Disabled"}</p>
        <p class="mb-2">
          Global Cooldown: <input
            type="number"
            bind:value={reward.global_cooldown_seconds}
            class="bg-gray-700 text-white p-1 rounded"
          /> seconds
        </p>
        <button
          class="mt-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          onclick={() => (reward.is_enabled = !reward.is_enabled)}
        >
          {reward.is_enabled ? "Disable" : "Enable"}
        </button>
      </div>
    {/each}
    <button
      class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
      onclick={() => {
        currentCustomRewards = [
          ...currentCustomRewards,
          {
            id: `new_reward_${Date.now()}`,
            title: "New Reward",
            prompt: "",
            cost: 100,
            is_enabled: true,
            global_cooldown_seconds: 0,
            is_global_cooldown_enabled: false,
          },
        ];
        touched = true;
      }}
    >
      Add Reward
    </button>
    {#if touched}
      <button
        class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
        onclick={() => {
          info(
            `Saving custom rewards: ${JSON.stringify(currentCustomRewards)}`,
          );
          $serverConnection?.send({
            type: "twitchTrigger",
            UpdateCustomRewards: {
              rewards: currentCustomRewards.map((reward) => ({
                title: reward.title,
                prompt: reward.prompt,
                cost: reward.cost,
                is_enabled: reward.is_enabled,
                is_global_cooldown_enabled: reward.global_cooldown_seconds > 0,
                global_cooldown_seconds: reward.global_cooldown_seconds,
              })),
              request_id: $serverConnection?.getNextRequestId(
                "Update Custom Rewards - Twitch Page",
              ),
            },
          });
          touched = false;
        }}
      >
        Save Changes
      </button>
    {/if}
  </div>
{/if}
