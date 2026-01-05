<script lang="ts">
    import * as Card from "$lib/components/ui/card";
    import * as Queue from "$lib/components/modifiable-queue/index";
    import * as Select from "$lib/components/ui/select";
    import type { RewardInstance } from "$lib/rewards/types";
    import type { TriggerInstance } from "$lib/triggers/types";
    import { Minus } from "@lucide/svelte";
    import RewardEditor from "./reward-editor.svelte";
    import TriggerEditor from "./trigger-editor.svelte";
    import Input from "./ui/input/input.svelte";
    import Label from "./ui/label/label.svelte";
    import {
        QueueCollapseState,
        setCollapseContext,
    } from "./modifiable-queue/context.svelte";
    import { rewards as rewardConstructors } from "$lib/task-parts";

    let {
        id,
        name = $bindable(),
        trigger = $bindable(),
        rewards = $bindable([]),
        deleteSelf,
    }: {
        id: string;
        name: string;
        trigger: TriggerInstance<any>;
        rewards: RewardInstance<any>[];
        deleteSelf: () => void;
    } = $props();

    setCollapseContext(
        new QueueCollapseState({
            open: () => true,
            setOpen: (_: boolean) => {},
        }),
    );
</script>

<Card.Root>
    <Card.Header>
        <div class="grid items-center gap-1.5 mb-4">
            <Label for="title-{id}">Title</Label>
            <div class="flex flex-row items-center">
                <Input
                    id="title-{id}"
                    type="text"
                    bind:value={name}
                    class="max-w-lg"
                />
                <Minus
                    size="20"
                    class="ml-2 cursor-pointer text-red-500 hover:text-red-600"
                    onclick={deleteSelf}
                />
            </div>
        </div>
    </Card.Header>
    <hr />
    <Card.Content>
        <h2 class="text-lg font-medium mb-4">Trigger</h2>
        <TriggerEditor bind:trigger />
    </Card.Content>
    <hr />
    <Card.Content>
        <h2 class="text-lg font-medium mb-4">Actions</h2>
        <Queue.Root>
            {#each rewards as _, i}
                <Queue.Item>
                    <RewardEditor
                        bind:reward={
                            () => rewards[i],
                            (r) => {
                                rewards[i] = r;
                                rewards = rewards;
                            }
                        }
                    />
                    <Queue.Controls
                        ondelete={() => {
                            rewards.splice(i, 1);
                            rewards = rewards;
                        }}
                        onduplicate={() => {
                            const ctor =
                                rewards[i].constructor as typeof RewardInstance;
                            const copy = new ctor(
                                structuredClone(rewards[i].params),
                            );
                            rewards.splice(i + 1, 0, copy);
                            rewards = rewards;
                        }}
                    />
                </Queue.Item>
            {/each}
            <Select.Root
                bind:value={
                    () => "",
                    (v) => {
                        const ctor = rewardConstructors[v]?.reward;

                        if (ctor)
                            rewards.push(new ctor({}) as RewardInstance<any>);
                        rewards = rewards;
                    }
                }
                type="single"
            >
                <Select.Trigger
                    class="mt-4 w-full max-w-lg text-muted-foreground"
                    >Add New Reward
                </Select.Trigger>
                <Select.Content align="start">
                    {#each Object.entries(rewardConstructors) as [id, rewardTypeOption]}
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
        </Queue.Root>
    </Card.Content>
</Card.Root>
