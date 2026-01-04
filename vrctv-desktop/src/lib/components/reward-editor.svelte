<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import type { RewardInstance } from "$lib/rewards/types";
    import { rewards } from "$lib/task-parts";

    let {
        reward = $bindable(),
    }: {
        reward: RewardInstance<any>;
    } = $props();

    const EditorComponent = $derived(rewards[reward.reward.id]?.editor);
</script>

<div class="grid items-center gap-1.5">
    <Label>Type</Label>
    <Select.Root
        bind:value={
            () => reward.reward.id,
            (v) => {
                const ctor = rewards[v].reward;

                if (ctor) reward = new ctor({}) ?? reward;
            }
        }
        type="single"
    >
        <Select.Trigger class="mb-4 w-full max-w-lg"
            >{reward.reward.title}</Select.Trigger
        >
        <Select.Content align="start">
            {#each Object.entries(rewards) as [id, rewardTypeOption]}
                <Select.Item
                    value={id}
                    class="hover:bg-accent hover:text-accent-foreground"
                >
                    <div class="flex flex-col">
                        <span>{rewardTypeOption.reward.title}</span>
                        <span class="text-sm text-muted-foreground">
                            {rewardTypeOption.reward.description}
                        </span>
                    </div>
                </Select.Item>
            {/each}
        </Select.Content>
    </Select.Root>

    {#if EditorComponent}
        <EditorComponent bind:reward />
    {/if}
</div>
