<script lang="ts">
  import {
    customRewardsStore,
    overlays,
    rewardStore,
    type Trigger,
  } from "$lib/stores";
  import { Minus, Plus, RefreshCcw } from "@lucide/svelte";
  import type { PageProps } from "../$types";
  import type { Avatar, Result } from "../../bindings";
  import TwitchFilterEditor from "../../components/twitchFilterEditor.svelte";
  import { serverConnection } from "$lib/websocket";
  import { onMount } from "svelte";
  import StreamlabsFilterEditor from "../../components/streamlabsFilterEditor.svelte";
  import ParameterEditor from "../../components/parameterEditor.svelte";

  let { data: rawData }: PageProps = $props();
  const data = rawData as Result<Avatar[], string>;

  let avatars = $derived(
    data.status === "ok"
      ? data.data.sort((a, b) => a.name.localeCompare(b.name))
      : [],
  );

  onMount(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Rewards Page",
        ),
      },
    });
  });
</script>

<div class="p-4 bg-gray-800 rounded">
  <div class="mb-4">
    Default Avatar: <select
      bind:value={$rewardStore.baseAvatarId}
      class="ml-2 p-1 bg-gray-700 text-white rounded"
    >
      {#each avatars as avatar}
        <option value={avatar.id}>{avatar.name} ({avatar.id})</option>
      {/each}
    </select>
  </div>
  <div>
    Default Parameters:
    <Plus
      class="inline ml-2 cursor-pointer hover:text-gray-300"
      onclick={() => {
        let i = 1;
        while ($rewardStore.baseParams.hasOwnProperty(`new_param_${i}`)) {
          i += 1;
        }
        $rewardStore.baseParams[`new_param_${i}`] = "value";
        $rewardStore = { ...$rewardStore };
      }}
    />
    {#each Object.entries($rewardStore.baseParams) as [key, value]}
      <div class="mt-2">
        <input
          type="text"
          class="p-1 bg-gray-700 text-white rounded w-128"
          onchange={(e) => {
            $rewardStore.baseParams[e.currentTarget.value] =
              $rewardStore.baseParams[key];
            delete $rewardStore.baseParams[key];
            $rewardStore = { ...$rewardStore };
          }}
          value={key}
        />
        :
        <input
          type="text"
          bind:value={$rewardStore.baseParams[key]}
          class="ml-2 p-1 bg-gray-700 text-white rounded w-32"
        />

        <Minus
          class="inline ml-2 cursor-pointer hover:text-gray-300"
          onclick={() => {
            delete $rewardStore.baseParams[key];
            $rewardStore = { ...$rewardStore };
          }}
        />
      </div>
    {/each}
  </div>
</div>

<!-- Use these types for the rewards -->
<h2 class="text-2xl font-bold mt-4 mb-2">
  Custom Rewards
  <button
    class="ml-4 px-2 py-1 bg-gray-600 text-white rounded hover:bg-gray-700"
    onclick={() => {
      $rewardStore.rewards.push({
        type: "avatar",
        setsAvatar: null,
        setParams: {},
        title: "New Reward",
        timeoutSeconds: 300,
        on: { type: "twitch", matches: [] },
      });
      $rewardStore = { ...$rewardStore };
    }}
  >
    Add Reward
  </button>
</h2>
{#each $rewardStore.rewards as reward, rewardId}
  <div class="mt-2">
    <div class="p-4 bg-gray-800 rounded">
      <div class="mb-2">
        Title:
        <input
          type="text"
          bind:value={reward.title}
          class="ml-2 p-1 bg-gray-700 text-white rounded w-128"
        />
        <Minus
          class="inline ml-2 cursor-pointer hover:text-gray-300"
          onclick={() => {
            $rewardStore.rewards.splice(rewardId, 1);
            $rewardStore = { ...$rewardStore };
          }}
        />
      </div>
      <div class="mb-2">
        Timeout (seconds):
        <input
          type="number"
          bind:value={reward.timeoutSeconds}
          class="ml-2 p-1 bg-gray-700 text-white rounded w-32"
        />
      </div>

      <div class="mb-2">
        Type:
        <select
          bind:value={reward.type}
          class="ml-2 p-1 bg-gray-700 text-white rounded"
          onchange={() => {
            if (reward.type === "avatar") {
              reward.setsAvatar = null;
              reward.setParams = {};
            } else if (reward.type === "overlay") {
              reward.overlay = -1;
              reward.show = true;
            }
            $rewardStore = { ...$rewardStore };
          }}
        >
          <option value="avatar">Avatar</option>
          <option value="overlay">Overlay</option>
        </select>
      </div>

      {#if reward.type === "avatar"}
        <div class="mb-2">
          Sets Avatar:
          <select
            bind:value={reward.setsAvatar}
            class="ml-2 p-1 bg-gray-700 text-white rounded"
          >
            <option value={null}>None</option>
            {#each avatars as avatar}
              <option value={avatar.id}>{avatar.name} ({avatar.id})</option>
            {/each}
          </select>
        </div>
        <div class="mb-2">
          Sets Parameters:
          <Plus
            class="inline ml-2 cursor-pointer hover:text-gray-300"
            onclick={() => {
              let i = 1;
              while (reward.setParams.hasOwnProperty(`new_param_${i}`)) {
                i += 1;
              }
              reward.setParams[`new_param_${i}`] = "value";
              $rewardStore = { ...$rewardStore };
            }}
          />
          {#each Object.entries(reward.setParams) as [key, value] (key)}
            <div class="mt-2">
              <ParameterEditor
                avatarId={reward.setsAvatar ?? $rewardStore.baseAvatarId ?? ""}
                param={key}
                {value}
                onChange={(param, val) => {
                  if (param !== key) {
                    delete reward.setParams[key];
                  }
                  reward.setParams[param] = val;
                  $rewardStore = { ...$rewardStore };
                }}
              />

              <Minus
                class="inline ml-2 cursor-pointer hover:text-gray-300"
                onclick={() => {
                  delete reward.setParams[key];
                  $rewardStore = { ...$rewardStore };
                }}
              />
            </div>
          {/each}
        </div>
      {:else if reward.type === "overlay"}
        <div class="mb-2">
          Overlay:
          <select
            bind:value={reward.overlay}
            class="ml-2 p-1 bg-gray-700 text-white rounded"
          >
            <option value={-1}>None</option>
            {#each $overlays as overlay}
              <option value={overlay.id}
                >{overlay.name} (ID: {overlay.id})</option
              >
            {/each}
          </select>
        </div>
        <div class="mb-2">
          Show Overlay:
          <input type="checkbox" bind:checked={reward.show} class="ml-2" />
        </div>
      {/if}
      <h3 class="text-xl font-bold mt-4 mb-2">
        Trigger On: <RefreshCcw
          class="inline ml-2 cursor-pointer hover:text-gray-300"
          onclick={() => {
            reward.on = {
              type: reward.on.type === "twitch" ? "streamlabs" : "twitch",
              matches: [],
            };
            $rewardStore = { ...$rewardStore };
          }}
        />
      </h3>
      {#if reward.on.type === "twitch"}
        <div class="mb-2">
          Type: Twitch <Plus
            class="inline ml-2 cursor-pointer hover:text-gray-300"
            onclick={() => {
              (reward.on as Extract<Trigger, { type: "twitch" }>).matches.push({
                type: "ChannelPoints",
              });
              $rewardStore = { ...$rewardStore };
            }}
          />
          {#each reward.on.matches as match, index (index)}
            <div class="mt-2 p-2 bg-gray-700 rounded">
              <TwitchFilterEditor
                {match}
                onchange={(newMatch) => {
                  reward.on.matches[index] = newMatch;
                  $rewardStore = { ...$rewardStore };
                }}
              />
              <Minus
                class="inline ml-2 cursor-pointer hover:text-gray-300"
                onclick={() => {
                  reward.on.matches.splice(index, 1);
                  $rewardStore = { ...$rewardStore };
                }}
              />
            </div>
          {/each}
        </div>
      {:else if reward.on.type === "streamlabs"}
        <div class="mb-2">
          Type: Streamlabs <Plus
            class="inline ml-2 cursor-pointer hover:text-gray-300"
            onclick={() => {
              (
                reward.on as Extract<Trigger, { type: "streamlabs" }>
              ).matches.push({
                type: "donation",
              });
              $rewardStore = { ...$rewardStore };
            }}
          />
          {#each reward.on.matches as match, index (index)}
            <div class="mt-2 p-2 bg-gray-700 rounded">
              <StreamlabsFilterEditor
                {match}
                onchange={(newMatch) => {
                  reward.on.matches[index] = newMatch;
                  $rewardStore = { ...$rewardStore };
                }}
              />
              <Minus
                class="inline ml-2 cursor-pointer hover:text-gray-300"
                onclick={() => {
                  reward.on.matches.splice(index, 1);
                  $rewardStore = { ...$rewardStore };
                }}
              />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/each}
