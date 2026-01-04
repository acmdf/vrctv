<script lang="ts">
  import { rewardStore } from "$lib/stores/rewards";
  import type { PageProps } from "../$types";
  import type { Avatar, Result } from "../../bindings";
  import { serverConnection } from "$lib/websocket";
  import * as Card from "$lib/components/ui/card/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import AvatarSelector from "$lib/components/avatarSelector.svelte";
  import { TwitchWhisperTrigger } from "$lib/triggers/twitch-whisper";
  import TaskEditor from "$lib/components/task-editor.svelte";

  const { data: rawData }: PageProps = $props();
  const data = rawData as Result<Avatar[], string>;

  const avatars = $derived(
    data.status === "ok"
      ? data.data.sort((a, b) => a.name.localeCompare(b.name))
      : [],
  );

  $effect(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Rewards Page",
        ),
      },
    });
  });
</script>

<Card.Root class="w-fit">
  <Card.Header>
    <Card.Title class="text-3xl font-bold mb-4">Default Settings</Card.Title>
  </Card.Header>
  <Card.Content>
    <p class="mb-4">
      These settings will be applied to all custom rewards unless overridden
      below.
    </p>

    <AvatarSelector
      label="Default Avatar"
      bind:avatarId={$rewardStore.baseAvatarId}
      {avatars}
    />
  </Card.Content>
</Card.Root>

<!-- Use these types for the rewards -->
<h2 class="text-2xl font-bold mt-4 mb-2">
  Custom Rewards
  <Button
    class="ml-4 px-2 py-1 bg-gray-600 text-white rounded hover:bg-gray-700"
    onclick={() => {
      $rewardStore.tasks.push({
        id: crypto.randomUUID(),
        name: "New Reward",
        trigger: new TwitchWhisperTrigger({}),
        rewards: [],
      });
      $rewardStore = { ...$rewardStore };
    }}
  >
    Add Reward
  </Button>
</h2>
<div class="grid lg:grid-cols-2 2xl:grid-cols-3 gap-2">
  {#each $rewardStore.tasks as task (task.id)}
    <TaskEditor
      id={task.id}
      bind:name={task.name}
      bind:trigger={task.trigger}
      bind:rewards={task.rewards}
      deleteSelf={() => {
        $rewardStore.tasks = $rewardStore.tasks.filter((t) => t.id !== task.id);
        $rewardStore = { ...$rewardStore };
      }}
    />
  {/each}
</div>
