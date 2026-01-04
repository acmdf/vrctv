<script lang="ts">
    import type { RewardInstance } from "$lib/rewards/types";
    import Label from "../ui/label/label.svelte";
    import Input from "../ui/input/input.svelte";
    import {
        CancelOSCReward,
        type CancelOSCRewardParams,
    } from "$lib/rewards/cancel-osc";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof CancelOSCReward)) {
            reward = new CancelOSCReward({});
        }
    });

    let rewardParams: CancelOSCRewardParams = $derived(reward.params);

    function updateParams<T extends keyof CancelOSCRewardParams>(
        field: T,
        value: CancelOSCRewardParams[T],
    ) {
        rewardParams[field] = value;
        reward.params = rewardParams;
        reward = reward;
    }
</script>

<div class="grid items-center max-w-lg">
    <div class="grid items-center gap-1.5 mb-2">
        <Label>Channel</Label>
        <Input
            bind:value={
                () => reward.params.channel_id,
                (c) => updateParams("channel_id", c)
            }
            placeholder="Channel ID"
        />
    </div>
</div>
