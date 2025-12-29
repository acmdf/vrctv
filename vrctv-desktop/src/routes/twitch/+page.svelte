<script>
  import { customRewardsStore } from "$lib/stores";
  import { serverConnection } from "$lib/websocket";
  import { info } from "@tauri-apps/plugin-log";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import Label from "$lib/components/ui/label/label.svelte";
  import { Check, X } from "@lucide/svelte";

  let lastCustomRewardsState = $state($customRewardsStore);
  let currentCustomRewards = $state($customRewardsStore);
  let touched = $state(false);

  $effect(() => {
    if (
      JSON.stringify($customRewardsStore) !==
      JSON.stringify(lastCustomRewardsState)
    ) {
      info(`Custom rewards changed: ${JSON.stringify($customRewardsStore)}`);

      currentCustomRewards = $customRewardsStore;
      lastCustomRewardsState = $customRewardsStore;
      touched = false;
    } else if (
      !touched &&
      JSON.stringify(currentCustomRewards) !==
        JSON.stringify(lastCustomRewardsState)
    ) {
      touched = true;
    }
  });

  $effect(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Twitch Page",
        ),
      },
    });
  });
</script>

<h1 class="text-3xl font-bold mb-4">Twitch Custom Rewards</h1>
{#if touched}
  <div class="mb-4 p-4 bg-yellow-200 text-yellow-800 rounded">
    You have unsaved changes. Please save to apply them.
  </div>
{/if}

{#if Object.keys(currentCustomRewards).length === 0}
  <div class="p-4 dark:bg-gray-800 bg-gray-300 rounded">No custom rewards found.</div>
{:else}
  <div class="grid gap-6 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 mb-4">
    {#each Object.entries(currentCustomRewards) as [rewardId, reward]}
      <Card.Root>
        <Card.Header>
          <Card.Title>
            <div class="grid items-center gap-1.5">
              <Label for="title-{rewardId}">Title (visible to viewers)</Label>
              <div class="flex items-center w-full justify-between">
                <Input
                  type="text"
                  bind:value={reward.title}
                  class="flex-1"
                  id="title-{rewardId}"
                />
                {#if reward.is_enabled}
                  <Check class="ml-2 inline text-green-500" size="32" />
                {:else}
                  <X class="ml-2 inline text-red-500" size="32" />
                {/if}
              </div>
            </div>
          </Card.Title>
        </Card.Header>
        <hr />
        <Card.Content>
          <div class="flex flex-col space-y-4 mb-4">
            <div class="grid w-full items-center gap-1.5">
              <Label for="prompt-{rewardId}">Prompt (visible to viewers)</Label>
              <Input
                type="text"
                bind:value={reward.prompt}
                id="prompt-{rewardId}"
              />
            </div>
            <div class="grid w-full items-center gap-1.5">
              <Label for="cost-{rewardId}">Cost</Label>
              <InputGroup.Root>
                <InputGroup.Input
                  type="number"
                  bind:value={reward.cost}
                  id="cost-{rewardId}"
                />
                <InputGroup.Addon align="inline-end">
                  <InputGroup.Text>points</InputGroup.Text>
                </InputGroup.Addon>
              </InputGroup.Root>
            </div>
            <div class="grid w-full items-center gap-1.5">
              <Label for="global-cooldown-{rewardId}">Global Cooldown</Label>
              <InputGroup.Root>
                <InputGroup.Input
                  type="number"
                  bind:value={reward.global_cooldown_seconds}
                  id="global-cooldown-{rewardId}"
                />
                <InputGroup.Addon align="inline-end">
                  <InputGroup.Text>seconds</InputGroup.Text>
                </InputGroup.Addon>
              </InputGroup.Root>
            </div>
          </div>
        </Card.Content>

        <Card.Footer>
          <Button
            class="w-full"
            variant={reward.is_enabled ? "destructive" : "default"}
            onclick={() => (reward.is_enabled = !reward.is_enabled)}
          >
            {reward.is_enabled ? "Disable" : "Enable"}
          </Button>
        </Card.Footer>
      </Card.Root>
    {/each}
  </div>
  <div class="flex items-center justify-between space-x-4 w-full">
    <Button
      class="flex-1"
      onclick={() => {
        currentCustomRewards = [
          ...currentCustomRewards,
          {
            id: `new_reward_${Date.now()}`,
            title: "New Reward",
            prompt: "",
            cost: 100,
            is_enabled: true,
            global_cooldown_seconds: 0,
            is_global_cooldown_enabled: false,
          },
        ];
        touched = true;
      }}
    >
      Add Reward
    </Button>
    {#if touched}
      <Button
        class="flex-1"
        onclick={() => {
          info(
            `Saving custom rewards: ${JSON.stringify(currentCustomRewards)}`,
          );
          $serverConnection?.send({
            type: "twitchTrigger",
            UpdateCustomRewards: {
              rewards: currentCustomRewards.map((reward) => ({
                title: reward.title,
                prompt: reward.prompt,
                cost: reward.cost,
                is_enabled: reward.is_enabled,
                is_global_cooldown_enabled: reward.global_cooldown_seconds > 0,
                global_cooldown_seconds: reward.global_cooldown_seconds,
              })),
              request_id: $serverConnection?.getNextRequestId(
                "Update Custom Rewards - Twitch Page",
              ),
            },
          });
          touched = false;
        }}
        variant="secondary"
      >
        Save Changes
      </Button>
    {/if}
  </div>
{/if}
