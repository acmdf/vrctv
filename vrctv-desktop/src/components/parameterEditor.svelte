<script lang="ts">
    import { commands } from "../bindings";

    let {
        avatarId,
        param,
        value,
        onChange,
    }: {
        avatarId: string;
        param: string;
        value: string;
        onChange: (param: string, value: string) => void;
    } = $props();

    let paramsLoad = $derived(await commands.fetchAvatarOsc(avatarId));

    function removeUselessParams(params: string[]): string[] {
        return params.filter((p) => !p.endsWith("/LastSynced") && !p.startsWith("FT/v2") && !p.includes("SyncData") && ["_Squish", "_Stretch", "_Angle", "_IsPosed", "_IsGrabbed"].every((uselessEnding) => !p.endsWith(uselessEnding)));
    }
</script>

{#if paramsLoad.status === "ok"}
    <select
        value={param}
        onchange={(e) => {
            onChange((e.currentTarget as HTMLSelectElement).value, value);
        }}
        class="p-2 bg-gray-700 text-white rounded w-1/3 mr-2 mb-2"
    >
        {#each removeUselessParams(paramsLoad.data) as param}
            <option value={`/avatar/parameters/${param}`}>{param}</option>
        {/each}
    </select>
{:else}
    <p class="text-red-500">Error: {paramsLoad.error}</p>
{/if}
:
<input
    type="text"
    value={value}
    oninput={(e) => {
        onChange(param, (e.currentTarget as HTMLInputElement).value);
    }}
    class="ml-2 p-1 bg-gray-700 text-white rounded w-32"
/>
