<script lang="ts">
    import { cachedAvatarStore } from "$lib/avatar-list-cache";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as InputGroup from "$lib/components/ui/input-group/index.js";
    import {
        SetAvatarReward,
        type SetAvatarRewardParams,
    } from "$lib/rewards/set-avatar";
    import type { RewardInstance } from "$lib/rewards/types";
    import AvatarSelector from "../avatar-selector.svelte";
    import Label from "../ui/label/label.svelte";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof SetAvatarReward)) {
            reward = new SetAvatarReward({});
        }
    });

    let rewardParams: SetAvatarRewardParams = $derived(reward.params);

    function updateParams<T extends keyof SetAvatarRewardParams>(
        field: T,
        value: SetAvatarRewardParams[T],
    ) {
        rewardParams[field] = value;
        reward.params = rewardParams;
        reward = reward;
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
<div class="grid items-center gap-1.5 mb-2">
    <Label>Set Avatar</Label>
    <AvatarSelector
        bind:avatarId={
            () => reward.params.avatar_id,
            (v) => {
                updateParams("avatar_id", v);
            }
        }
        avatars={$cachedAvatarStore}
    />
</div>
<div class="grid items-center gap-1.5">
    <Label>Return to</Label>
    <Tabs.Root
        bind:value={
            () => reward.params.return_to,
            (newType) => updateParams("return_to", newType)
        }
    >
        <Tabs.List>
            <Tabs.Trigger value="default">Default</Tabs.Trigger>
            <Tabs.Trigger value="previous">Previous</Tabs.Trigger>
            <Tabs.Trigger value="specific">Specific</Tabs.Trigger>
        </Tabs.List>
    </Tabs.Root>
    {#if reward.params.return_to === "specific"}
        Return Avatar
        <AvatarSelector
            bind:avatarId={
                () => reward.params.return_avatar_id,
                (v) => updateParams("return_avatar_id", v)
            }
            avatars={$cachedAvatarStore}
        />
    {/if}
</div>
