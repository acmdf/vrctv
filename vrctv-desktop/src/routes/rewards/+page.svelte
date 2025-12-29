<script lang="ts">
  import {
    overlays,
    rewardStore,
    type Reward,
    type Trigger,
  } from "$lib/stores";
  import { Minus, Plus, RefreshCcw } from "@lucide/svelte";
  import type { PageProps } from "../$types";
  import type { Avatar, Result } from "../../bindings";
  import TwitchFilterEditor from "$lib/components/twitchFilterEditor.svelte";
  import { serverConnection } from "$lib/websocket";
  import StreamlabsFilterEditor from "$lib/components/streamlabsFilterEditor.svelte";
  import ParameterEditor from "$lib/components/parameterEditor.svelte";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as Tabs from "$lib/components/ui/tabs";
  import Label from "$lib/components/ui/label/label.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import AvatarSelector from "$lib/components/avatarSelector.svelte";
  import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";
  import type { StreamLabsEventMatcher } from "$lib/streamlabs";

  const { data: rawData }: PageProps = $props();
  const data = rawData as Result<Avatar[], string>;

  const avatars = $derived(
    data.status === "ok"
      ? data.data.sort((a, b) => a.name.localeCompare(b.name))
      : []
  );

  $effect(() => {
    $serverConnection?.send({
      type: "twitchTrigger",
      GetCustomRewards: {
        request_id: $serverConnection?.getNextRequestId(
          "Get Custom Rewards - Rewards Page"
        ),
      },
    });
  });

  function typeName(type: Reward["type"]) {
    return {
      avatar: "Set Avatar",
      overlay: "Set Overlay State",
      avatarCancel: "Cancel the current Avatar",
      overlayCancel: "Cancel a certain Overlay",
    }[type];
  }

  function overlayName(overlay: number) {
    return $overlays.find((o) => o.id == overlay) ?? "Select Overlay";
  }
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

    <div class="grid items-center max-w-lg">
      <Label>Default Parameters</Label>
      {#each Object.entries($rewardStore.baseParams) as [param, value]}
        <ParameterEditor
          avatarId={$rewardStore.baseAvatarId ?? ""}
          {param}
          {value}
          onChange={(newParam, val) => {
            const baseParams = $rewardStore.baseParams;

            if (param !== newParam) {
              delete baseParams[param];
            }

            if (newParam) {
              baseParams[newParam] = val;
            }

            rewardStore.set({
              ...$rewardStore,
              baseParams: {
                ...baseParams,
              },
            });
          }}
        />
      {/each}
      <ParameterEditor
        avatarId={$rewardStore.baseAvatarId ?? ""}
        placeholder={true}
        param=""
        value=""
        onChange={(param, val) => {
          if (!param) return;

          rewardStore.set({
            ...$rewardStore,
            baseParams: {
              ...$rewardStore.baseParams,
              [param]: val,
            },
          });
        }}
      />
    </div>
  </Card.Content>
</Card.Root>

<!-- Use these types for the rewards -->
<h2 class="text-2xl font-bold mt-4 mb-2">
  Custom Rewards
  <Button
    class="ml-4 px-2 py-1 bg-gray-600 text-white rounded hover:bg-gray-700"
    onclick={() => {
      $rewardStore.rewards.push({
        type: "avatar",
        setsAvatar: undefined,
        setParams: {},
        title: "New Reward",
        timeoutSeconds: 300,
        on: { type: "twitch", matches: [] },
      });
      $rewardStore = { ...$rewardStore };
    }}
  >
    Add Reward
  </Button>
</h2>
<div class="grid lg:grid-cols-2 xl:grid-cols-3 gap-2">
  {#each $rewardStore.rewards as reward, rewardId}
    <Card.Root class="mb-4">
      <Card.Header>
        <div class="grid items-center gap-1.5 mb-4">
          <Label for="title-{rewardId}">Title</Label>
          <Input
            id="title-{rewardId}"
            type="text"
            bind:value={reward.title}
            class="max-w-lg"
          />
        </div>
        <div class="grid items-center gap-1.5">
          <Label>Type</Label>
          <Select.Root
            bind:value={reward.type}
            type="single"
            onValueChange={(_) => {
              if (reward.type === "avatar") {
                reward.setsAvatar = undefined;
                reward.setParams = {};
                reward.timeoutSeconds = 300;
              } else if (reward.type === "overlay") {
                reward.overlay = -1;
                reward.show = true;
                reward.timeoutSeconds = 300;
              } else if (reward.type === "overlayCancel") {
                reward.overlay = -1;
              }
              $rewardStore = { ...$rewardStore };
            }}
          >
            <Select.Trigger class="mb-4 w-full max-w-lg">
              {typeName(reward.type)}
            </Select.Trigger>
            <Select.Content align="start">
              <Select.Group>
                <Select.Label>Control Avatar</Select.Label>
                <Select.Item value="avatar">Set Avatar</Select.Item>
                <Select.Item value="avatarCancel">Cancel Avatar</Select.Item>
              </Select.Group>
              <Select.Group>
                <Select.Label>Control Overlays</Select.Label>
                <Select.Item value="overlay">Set Overlay State</Select.Item>
                <Select.Item value="overlayCancel">Cancel Overlay</Select.Item>
              </Select.Group>
            </Select.Content>
          </Select.Root>
        </div>
        <Button
          variant="destructive"
          class="w-full"
          onclick={() => {
            $rewardStore.rewards.splice(rewardId, 1);

            $rewardStore = { ...$rewardStore };
          }}
        >
          Delete
        </Button>
      </Card.Header>
      <hr />
      <Card.Content>
        {#if reward.type === "avatar" || reward.type === "overlay"}
          <div class="grid items-center gap-1.5 mb-4">
            <Label for="timeout-{rewardId}">Timeout</Label>
            <InputGroup.Root class="w-full max-w-lg">
              <InputGroup.Input
                id="timeout-{rewardId}"
                type="number"
                bind:value={reward.timeoutSeconds}
              />
              <InputGroup.Addon align="inline-end">
                <InputGroup.Text>seconds</InputGroup.Text>
              </InputGroup.Addon>
            </InputGroup.Root>
          </div>
        {/if}
        {#if reward.type === "avatar"}
          <AvatarSelector
            label="Sets Avatar"
            bind:avatarId={reward.setsAvatar}
            {avatars}
          />

          <div class="grid items-center max-w-lg">
            <Label>Sets Paramaters</Label>
            {#each Object.entries(reward.setParams) as [param, value]}
              <ParameterEditor
                avatarId={reward.setsAvatar ?? $rewardStore.baseAvatarId ?? ""}
                {param}
                {value}
                onChange={(newParam, val) => {
                  if (param !== newParam) {
                    delete reward.setParams[param];
                  }

                  if (newParam) {
                    reward.setParams[newParam] = val;
                  }

                  $rewardStore = { ...$rewardStore };
                }}
              />
            {/each}
            <ParameterEditor
              avatarId={reward.setsAvatar ?? $rewardStore.baseAvatarId ?? ""}
              placeholder={true}
              param=""
              value=""
              onChange={(param, val) => {
                if (!param) return;

                reward.setParams[param] = val;

                $rewardStore = { ...$rewardStore };
              }}
            />
          </div>
        {:else if reward.type === "overlay"}
          <div class="grid items-center gap-1.5">
            <Label>Overlay</Label>
            <div class="max-w-lg flex flex-row items-center space-x-2">
              <Select.Root
                bind:value={
                  () => reward.overlay.toString(),
                  (v) => (reward.overlay = parseInt(v))
                }
                type="single"
              >
                <Select.Trigger class="w-full">
                  {overlayName(reward.overlay)}
                </Select.Trigger>
                <Select.Content align="start">
                  {#each $overlays as overlay}
                    <Select.Item value={overlay.id.toString()}>
                      {overlay.name} (ID: {overlay.id})
                    </Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
              <Tabs.Root
                bind:value={
                  () => (reward.show ? "show" : "hide"),
                  (v) => (reward.show = v === "show")
                }
              >
                <Tabs.List>
                  <Tabs.Trigger value="show">Show</Tabs.Trigger>
                  <Tabs.Trigger value="hide">Hide</Tabs.Trigger>
                </Tabs.List>
              </Tabs.Root>
            </div>
          </div>
        {:else if reward.type === "overlayCancel"}
          <div class="grid items-center gap-1.5">
            <Label>Cancel Overlay</Label>
            <Select.Root
              bind:value={
                () => reward.overlay.toString(),
                (v) => (reward.overlay = parseInt(v))
              }
              type="single"
            >
              <Select.Trigger class="w-full max-w-lg">
                {overlayName(reward.overlay)}
              </Select.Trigger>
              <Select.Content align="start">
                {#each $overlays as overlay}
                  <Select.Item value={overlay.id.toString()}>
                    {overlay.name} (ID: {overlay.id})
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
        {/if}
      </Card.Content>
      <hr />
      <Card.Content>
        <h2 class="text-xl font-bold mb-4">Reward Triggers</h2>
        <div class="grid items-center gap-1.5 mb-4">
          <Label>Source</Label>
          <Tabs.Root bind:value={reward.on.type}>
            <Tabs.List>
              <Tabs.Trigger value="twitch">Twitch</Tabs.Trigger>
              <Tabs.Trigger value="streamlabs">Streamlabs</Tabs.Trigger>
            </Tabs.List>
            <Tabs.Content value="twitch">
              <div class="mb-2 space-y-2">
                {#each reward.on.matches as match, index (index)}
                  <TwitchFilterEditor
                    match={match as Partial<TwitchEventSource>}
                    onchange={(newMatch) => {
                      if (newMatch === undefined) {
                        reward.on.matches.splice(index, 1);
                      } else {
                        reward.on.matches[index] = newMatch;
                      }

                      $rewardStore = { ...$rewardStore };
                    }}
                  />
                {/each}

                <TwitchFilterEditor
                  placeholder={true}
                  onchange={(newMatch) => {
                    if (newMatch === undefined) return;

                    (reward.on.matches as Partial<TwitchEventSource>[]).push(
                      newMatch
                    );
                    $rewardStore = { ...$rewardStore };
                  }}
                />
              </div>
            </Tabs.Content>
            <Tabs.Content value="streamlabs">
              <div class="mb-2 space-y-2">
                {#each reward.on.matches as match, index (index)}
                  <StreamlabsFilterEditor
                    match={match as StreamLabsEventMatcher}
                    onchange={(newMatch) => {
                      if (newMatch === undefined) {
                        reward.on.matches.splice(index, 1);
                      } else {
                        reward.on.matches[index] = newMatch;
                      }

                      $rewardStore = { ...$rewardStore };
                    }}
                  />
                {/each}

                <StreamlabsFilterEditor
                  placeholder={true}
                  onchange={(newMatch) => {
                    if (newMatch === undefined) return;

                    (reward.on.matches as StreamLabsEventMatcher[]).push(
                      newMatch
                    );
                    $rewardStore = { ...$rewardStore };
                  }}
                />
              </div>
            </Tabs.Content>
          </Tabs.Root>
        </div>
      </Card.Content>
    </Card.Root>
  {/each}
</div>
