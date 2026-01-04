<script lang="ts">
    import * as Card from "$lib/components/ui/card";
    import type { RewardInstance } from "$lib/rewards/types";
    import type { TriggerInstance } from "$lib/triggers/types";
    import { Minus } from "@lucide/svelte";
    import RewardEditor from "./reward-editor.svelte";
    import TriggerEditor from "./trigger-editor.svelte";
    import Input from "./ui/input/input.svelte";
    import Label from "./ui/label/label.svelte";

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
        {#each rewards as _, i}
            <RewardEditor
                bind:reward={
                    () => rewards[i],
                    (r) => {
                        rewards[i] = r;
                        rewards = rewards;
                    }
                }
            />
        {/each}
    </Card.Content>
</Card.Root>
