<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import * as InputGroup from "$lib/components/ui/input-group/index.js";
    import type { RewardInstance } from "$lib/rewards/types";
    import AvatarSelector from "../avatarSelector.svelte";
    import {
        SetOSCReward,
        type SetOSCRewardParams,
    } from "$lib/rewards/set-osc";
    import Label from "../ui/label/label.svelte";
    import ParameterEditor from "../parameterEditor.svelte";
    import { cachedAvatarStore } from "$lib/avatar-list-cache";
    import type { KV } from "$lib/triggers/types";
    import Input from "../ui/input/input.svelte";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof SetOSCReward)) {
            reward = new SetOSCReward({ id: crypto.randomUUID() });
        }
    });

    let rewardParams: SetOSCRewardParams = $derived({ ...reward.params });

    let avatarId = $derived(reward.params.for_avatar);
    let params = $derived({ ...reward.params.params });
    let returnParams = $derived({ ...reward.params.return_params });

    function updateParams<T extends keyof SetOSCRewardParams>(
        field: T,
        value: SetOSCRewardParams[T],
    ) {
        reward.params[field] = value;
        reward = reward;
    }
</script>

<div class="grid items-center gap-1.5 mb-2">
    <Label>Channel</Label>
    <Input
        bind:value={
            () => reward.params.channel_id, (c) => updateParams("channel_id", c)
        }
        placeholder="Channel ID"
    />
</div>
<AvatarSelector
    label="For Avatar"
    bind:avatarId={() => avatarId, (v) => updateParams("for_avatar", v)}
    avatars={$cachedAvatarStore}
/>
<div class="grid items-center gap-1.5 mb-2">
    <Label>Sets Parameters</Label>
    {#each Object.entries(params as KV) as [param, value] (param)}
        <ParameterEditor
            {avatarId}
            bind:param={
                () => param,
                (newParam) => {
                    if (param !== newParam) {
                        delete params[param];
                    }

                    if (newParam) {
                        params[newParam] = value;
                    }

                    updateParams("params", params);
                }
            }
            bind:value={
                () => value,
                (val) => {
                    params[param] = val;
                    updateParams("params", params);
                }
            }
        />
    {/each}
    <ParameterEditor
        {avatarId}
        placeholder="Set this parameter"
        bind:param={
            () => "",
            (newParam) => {
                if (!newParam) return;

                params[newParam] = "";
                updateParams("params", params);
            }
        }
    />
</div>
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
    <Label>Return to</Label>
    <Tabs.Root
        bind:value={
            () => reward.params.return_to,
            (newType) => updateParams("return_to", newType)
        }
    >
        <Tabs.List>
            <Tabs.Trigger value="previous">Previous</Tabs.Trigger>
            <Tabs.Trigger value="specific">Specific</Tabs.Trigger>
        </Tabs.List>
    </Tabs.Root>
</div>
{#if reward.params.return_to === "specific"}
    <div class="grid items-center gap-1.5">
        <Label>Returns to Parameters</Label>
        {#each Object.entries(returnParams as KV) as [param, value] (param)}
            <ParameterEditor
                avatarId={avatarId}
                bind:param={
                    () => param,
                    (newParam) => {
                        if (param !== newParam) {
                            delete returnParams[param];
                        }

                        if (newParam) {
                            returnParams[newParam] = value;
                        }

                        updateParams(
                            "return_params",
                            returnParams,
                        );
                    }
                }
                bind:value={
                    () => value,
                    (val) => {
                        returnParams[param] = val;

                        updateParams(
                            "return_params",
                            returnParams,
                        );
                    }
                }
            />
        {/each}
        <ParameterEditor
            avatarId={avatarId}
            placeholder="Set this parameter"
            bind:param={
                () => "",
                (newParam) => {
                    if (!newParam) return;

                    returnParams[newParam] = "";
                    updateParams("return_params", returnParams);
                }
            }
        />
    </div>
{/if}