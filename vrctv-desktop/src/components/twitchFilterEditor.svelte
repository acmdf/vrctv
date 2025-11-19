<script lang="ts">
    import { customRewardsStore } from "$lib/stores";
    import type { TwitchEventSource } from "../../../vrctv-common/bindings/TwitchEventSource";

    const {
        match,
        onchange,
    }: {
        match: Partial<TwitchEventSource>;
        onchange: (newMatch: Partial<TwitchEventSource>) => void;
    } = $props();

    const twitchFilterDemoObjects: TwitchEventSource[] = [
        {
            type: "ChannelPoints",
            reward_id: "12345",
            reward_name: "Test Reward",
        },
        {
            type: "BitDonation",
            amount: 100,
            message: "Great stream!",
            emojis: ["Kappa"],
        },
        { type: "Whisper", message: "Hello there!" },
        { type: "Message", message: "This is a chat message." },
    ];
</script>

Type: <select
    value={match.type}
    onchange={(e) => {
        onchange({
            type: e.currentTarget.value as TwitchEventSource["type"],
        });
    }}
    class="p-1 bg-gray-600 text-white rounded"
>
    {#each twitchFilterDemoObjects as demo}
        <option value={demo.type}>{demo.type}</option>
    {/each}
</select>

{#if match.type === "ChannelPoints"}
    <select
        value={match.reward_id ?? ""}
        onchange={(e) => {
            if (e.currentTarget.value == "") {
                let { reward_id, ...rest } = match;
                onchange(rest);
            } else {
                onchange({
                    ...match,
                    reward_id: e.currentTarget.value,
                });
            }
        }}
        class="p-1 bg-gray-600 text-white rounded ml-2"
    >
        <option value="">Any Reward</option>
        {#each $customRewardsStore as reward (reward.id)}
            <option value={reward.id}>{reward.title}</option>
        {/each}
    </select>
{:else if match.type === "BitDonation"}
    <input
        type="number"
        min="0"
        value={match.amount ?? ""}
        oninput={(e) => {
            const val = (e.currentTarget as HTMLInputElement).value;
            if (val === "") {
                let { amount, ...rest } = match;
                onchange(rest);
            } else {
                onchange({
                    ...match,
                    amount: parseInt(val),
                });
            }
        }}
        placeholder="Minimum Amount"
        class="p-1 bg-gray-600 text-white rounded ml-2 w-32"
    />
    <input
        type="text"
        value={match.message ?? ""}
        oninput={(e) => {
            const val = (e.currentTarget as HTMLInputElement).value;
            if (val === "") {
                let { message, ...rest } = match;
                onchange(rest);
            } else {
                onchange({
                    ...match,
                    message: val,
                });
            }
        }}
        placeholder="Message Contains"
        class="p-1 bg-gray-600 text-white rounded ml-2 w-64"
    />
{:else if match.type === "Whisper" || match.type === "Message"}
    <input
        type="text"
        value={match.message ?? ""}
        oninput={(e) => {
            const val = (e.currentTarget as HTMLInputElement).value;
            if (val === "") {
                let { message, ...rest } = match;
                onchange(rest);
            } else {
                onchange({
                    ...match,
                    message: val,
                });
            }
        }}
        placeholder="Message Contains"
        class="p-1 bg-gray-600 text-white rounded ml-2 w-64"
    />
{/if}
