<script lang="ts">
    import type { RewardInstance } from "$lib/rewards/types";
    import Label from "../ui/label/label.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import * as InputGroup from "$lib/components/ui/input-group/index.js";
    import {
        SetOverlayReward,
        type SetOverlayRewardParams,
    } from "$lib/rewards/set-overlay";
    import { overlays } from "$lib/stores/overlays";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof SetOverlayReward)) {
            reward = new SetOverlayReward({});
        }
    });

    let rewardParams: SetOverlayRewardParams = $derived(reward.params);

    function updateParams<T extends keyof SetOverlayRewardParams>(
        field: T,
        value: SetOverlayRewardParams[T],
    ) {
        rewardParams[field] = value;
        reward.params = rewardParams;
        reward = reward;
    }

    function overlayName(overlay: number) {
        return $overlays.find((o) => o.id == overlay)?.name ?? "Select Overlay";
    }
</script>

<div class="grid items-center gap-1.5 mb-2">
    <Label>Timeout</Label>
    <InputGroup.Root class="w-full max-w-lg">
        <InputGroup.Input
            type="number"
            bind:value={
                () => rewardParams.timeout_ms / 1000,
                (v) => updateParams("timeout_ms", v * 1000)
            }
        />
        <InputGroup.Addon align="inline-end">
            <InputGroup.Text>seconds</InputGroup.Text>
        </InputGroup.Addon>
    </InputGroup.Root>
</div>
<div class="grid items-center gap-1.5">
    <Label>Overlay</Label>
    <div class="max-w-lg flex flex-row items-center space-x-2">
        <Select.Root
            bind:value={
                () => rewardParams.overlay_id.toString(),
                (v) => updateParams("overlay_id", parseInt(v))
            }
            type="single"
        >
            <Select.Trigger class="w-full">
                {overlayName(reward.params.overlay_id)}
            </Select.Trigger>
            <Select.Content align="start">
                {#each $overlays as overlay}
                    <Select.Item value={overlay.id.toString()}>
                        {overlay.name} (ID: {overlay.id})
                    </Select.Item>
                {/each}
            </Select.Content>
        </Select.Root>
        <Tabs.Root
            bind:value={
                () => (reward.params.show ? "show" : "hide"),
                (v) => updateParams("show", v === "show")
            }
        >
            <Tabs.List>
                <Tabs.Trigger value="show">Show</Tabs.Trigger>
                <Tabs.Trigger value="hide">Hide</Tabs.Trigger>
            </Tabs.List>
        </Tabs.Root>
    </div>
</div>
