<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import type { TriggerInstance } from "$lib/triggers/types";
    import { TwitchChannelPointsTrigger } from "$lib/triggers/twitch-channel-points";
    import { customRewardsStore } from "$lib/stores/rewards";
    import Label from "../ui/label/label.svelte";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    let channelPointsTrigger = $derived.by(() => {
        if (trigger instanceof TwitchChannelPointsTrigger) {
            return trigger as TwitchChannelPointsTrigger;
        }

        trigger = new TwitchChannelPointsTrigger({});
        return trigger as TwitchChannelPointsTrigger;
    });

    function getRewardName(reward: string | undefined) {
        if (reward === "" || reward === undefined) return "Any Reward";

        return (
            $customRewardsStore.find((r) => r.id === reward)?.title ??
            "Unknown reward"
        );
    }
</script>

<Label>On</Label>
<Select.Root
    type="single"
    bind:value={
        () => trigger.params.reward_id ?? "",
        (newReward) => {
            if (newReward == "") {
                let { reward_id, ...rest } = channelPointsTrigger.params;
                channelPointsTrigger.params = rest;
            } else {
                channelPointsTrigger.params = {
                    ...channelPointsTrigger.params,
                    reward_id: newReward,
                };
            }
            trigger = channelPointsTrigger;
        }
    }
>
    <Select.Trigger>{getRewardName(trigger.params.reward_id)}</Select.Trigger>
    <Select.Content>
        <Select.Item value="">Any Reward</Select.Item>
        {#each $customRewardsStore as reward (reward.id)}
            <Select.Item value={reward.id}>{reward.title}</Select.Item>
        {/each}
    </Select.Content>
</Select.Root>
