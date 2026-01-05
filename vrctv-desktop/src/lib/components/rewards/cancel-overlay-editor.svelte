<script lang="ts">
    import type { RewardInstance } from "$lib/rewards/types";
    import Label from "../ui/label/label.svelte";
    import * as Select from "$lib/components/ui/select";

    import {
        CancelOverlayReward,
        type CancelOverlayRewardParams,
    } from "$lib/rewards/cancel-overlay";
    import { overlays } from "$lib/stores/overlays";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof CancelOverlayReward)) {
            reward = new CancelOverlayReward({});
        }
    });

    let rewardParams: CancelOverlayRewardParams = $derived(reward.params);

    function updateParams<T extends keyof CancelOverlayRewardParams>(
        field: T,
        value: CancelOverlayRewardParams[T],
    ) {
        rewardParams[field] = value;
        reward.params = rewardParams;
        reward = reward;
    }

    function overlayName(overlay: number | undefined) {
        if (overlay === undefined) {
            return "None (Cancel all overlays)";
        }

        return $overlays.find((o) => o.id == overlay)?.name ?? "Select Overlay";
    }
</script>

<div class="grid items-center gap-1.5 mb-2">
    <Label>Overlay</Label>
    <Select.Root
        bind:value={
            () =>
                reward.params.overlay_id
                    ? reward.params.overlay_id.toString()
                    : "",
            (v) => {
                if (v === "") {
                    updateParams("overlay_id", undefined);
                } else {
                    updateParams("overlay_id", parseInt(v));
                }
            }
        }
        type="single"
    >
        <Select.Trigger class="w-full">
            {overlayName(reward.params.overlay_id)}
        </Select.Trigger>
        <Select.Content align="start">
            <Select.Item value="">None (Cancel all overlays)</Select.Item>
            {#each $overlays as overlay}
                <Select.Item value={overlay.id.toString()}>
                    {overlay.name} (ID: {overlay.id})
                </Select.Item>
            {/each}
        </Select.Content>
    </Select.Root>
</div>
