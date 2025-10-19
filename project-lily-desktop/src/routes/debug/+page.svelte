<script lang="ts">
    import { onMount } from "svelte";
    import { commands } from "../../bindings";
    import {
        serviceStateStore,
        oscStateStore,
        clientStateStore,
        taskStateStore,
        rewardStore,
        customRewardsStore,
        eventLogStore,
        rewardQueue,currentReward
    } from "../../lib/stores";
    import type { PageProps } from "./$types";
    import { warn } from "@tauri-apps/plugin-log";
    import { serverConnection } from "$lib/websocket";

    let { data }: PageProps = $props();

    function formatValue(value: any): string {
        if (typeof value === "object") {
            return JSON.stringify(value, null, 2);
        }
        return String(value);
    }

    onMount(() => {
        $serverConnection?.send({
            type: "twitchTrigger",
            GetCustomRewards: {
                request_id: $serverConnection?.getNextRequestId(
                    "Get Custom Rewards - Debug Page",
                ),
            },
        });
    });

    warn(`Debug page data: ${JSON.stringify(data)}`);
</script>

<h1 class="text-3xl font-bold mb-4">Debug Information</h1>
<h2 class="text-2xl font-bold mb-2">Client Info</h2>
<div class="mb-4">
    {#each Object.entries($clientStateStore) as [key, value]}
        <div class="flex p-2 items-center">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{key}</p>
            {formatValue(value)}
        </div>
    {/each}
</div>
<h2 class="text-2xl font-bold mb-2">Task Info</h2>
<div class="mb-4">
    {#each Object.entries($taskStateStore) as [key, value]}
        <div class="flex p-2 items-center">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{key}</p>
            {formatValue(value)}
        </div>
    {/each}
</div>
<h2 class="text-2xl font-bold mb-2">Reward Store</h2>
<div class="mb-4">
    {#each Object.entries($rewardStore) as [key, value]}
        <div class="flex p-2 items-center">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{key}</p>
            {formatValue(value)}
        </div>
    {/each}
</div>

<h2 class="text-2xl font-bold mb-2">Custom Rewards</h2>
<div class="mb-4">
    {#each Object.entries($customRewardsStore) as [key, value]}
        <div class="flex p-2 items-center">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{key}</p>
            {formatValue(value)}
        </div>
    {/each}
</div>

<h2 class="text-2xl font-bold mb-2">Event Log</h2>
<div class="mb-4 max-h-64 overflow-y-auto">
    {#each $eventLogStore as event, index (index)}
        <div class="flex p-2 items-center border-b border-gray-700">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{index + 1}</p>
            {formatValue(event)}
        </div>
    {/each}
</div>

<h2 class="text-2xl font-bold mb-2">Current Queue</h2>
<div class="mb-4 max-h-64 overflow-y-auto">
    {#each $rewardQueue as reward, index (index)}
        <div class="flex p-2 items-center border-b border-gray-700">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{index + 1}</p>
            {formatValue(reward)}
        </div>
    {/each}
</div>
<h2 class="text-2xl font-bold mb-2">Current Reward</h2>
<div class="mb-4 max-h-64 overflow-y-auto">
    {#if $currentReward}
        <div class="flex p-2 items-center border-b border-gray-700">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">1</p>
            {formatValue($currentReward)}
        </div>
    {:else}
        <p class="text-gray-400">No current reward being processed.</p>
    {/if}
</div>

<h2 class="text-2xl font-bold mb-2">OSC State</h2>
<div class="mb-4">
    {#each Object.entries($oscStateStore) as [address, value]}
        <div class="flex p-2 items-center">
            <p class="text-md bg-gray-800 rounded p-2 mr-2">{address}</p>
            {formatValue(value)}
        </div>
    {/each}
</div>

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
        {#each data.data as avatar}
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
