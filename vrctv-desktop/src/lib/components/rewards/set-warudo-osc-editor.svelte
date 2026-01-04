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
    import {
        SetWarudoOscReward,
        type SetWarudoOscRewardParams,
    } from "$lib/rewards/set-warudo-osc";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    $effect(() => {
        if (!(reward instanceof SetWarudoOscReward)) {
            reward = new SetWarudoOscReward({});
        }
    });

    let rewardParams: SetWarudoOscRewardParams = $derived(reward.params);

    function updateParams<T extends keyof SetWarudoOscRewardParams>(
        field: T,
        value: SetWarudoOscRewardParams[T],
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
    <div class="grid items-center gap-1.5 mb-2">
        <Label>Sets Parameters</Label>
        {#each Object.entries(reward.params.params as KV) as [param, value]}
            <ParameterEditor
                {param}
                {value}
                onChange={(newParam, val) => {
                    if (param !== newParam) {
                        delete reward.params.params[param];
                    }

                    if (newParam) {
                        reward.params.params[newParam] = val;
                    }

                    updateParams("params", reward.params.params);
                }}
            />
        {/each}
        <ParameterEditor
            placeholder="Set this parameter"
            param=""
            value=""
            onChange={(param, val) => {
                if (!param) return;

                reward.params.params[param] = val;

                updateParams("params", reward.params.params);
            }}
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
    <div class="grid items-center gap-1.5">
        <Label>Returns to Parameters</Label>
        {#each Object.entries(reward.params.return_params as KV) as [param, value]}
            <ParameterEditor
                {param}
                {value}
                onChange={(newParam, val) => {
                    if (param !== newParam) {
                        delete reward.params.return_params[param];
                    }

                    if (newParam) {
                        reward.params.return_params[newParam] = val;
                    }

                    updateParams("return_params", reward.params.return_params);
                }}
            />
        {/each}
        <ParameterEditor
            placeholder="Set this parameter"
            param=""
            value=""
            onChange={(param, val) => {
                if (!param) return;

                reward.params.return_params[param] = val;

                updateParams("return_params", reward.params.return_params);
            }}
        />
    </div>
</div>
