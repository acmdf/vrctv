<script lang="ts">
    import type { StreamLabsEventMatcher } from "$lib/streamlabs";

    const {
        match,
        onchange,
    }: {
        match: StreamLabsEventMatcher;
        onchange: (newMatch: StreamLabsEventMatcher) => void;
    } = $props();

    const streamlabsFilterDemoObjects: StreamLabsEventMatcher[] = [
        {
            type: "donation",
            amount: 500,
            message: "MaidMode",
        },
    ];
</script>

Type: <select
    value={match.type}
    onchange={(e) => {
        onchange({
            type: e.currentTarget.value as StreamLabsEventMatcher["type"],
        });
    }}
    class="p-1 bg-gray-600 text-white rounded"
>
    {#each streamlabsFilterDemoObjects as demo}
        <option value={demo.type}>{demo.type}</option>
    {/each}
</select>

{#if match.type === "donation"}
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
{/if}
