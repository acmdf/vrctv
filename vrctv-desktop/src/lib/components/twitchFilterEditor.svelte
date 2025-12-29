<script lang="ts">
  import { customRewardsStore } from "$lib/stores";
  import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";
  import * as Card from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import Button from "./ui/button/button.svelte";
  import { Root } from "./ui/button";

  const {
    placeholder = false,
    match,
    onchange,
  }: {
    placeholder?: boolean;
    match?: Partial<TwitchEventSource>;
    onchange: (newMatch: Partial<TwitchEventSource> | undefined) => void;
  } = $props();

  const twitchFilterDemoObjects: TwitchEventSource[] = [
    {
      type: "ChannelPoints",
      reward_id: "12345",
      reward_name: "Test Reward",
    },
    {
      type: "BitDonation",
      amount: 100,
      message: "Great stream!",
      emojis: ["Kappa"],
    },
    { type: "Whisper", message: "Hello there!" },
    { type: "Message", message: "This is a chat message." },
  ];

  $effect(() => {
    if ((match?.type as string) === "") {
      onchange(undefined);
    }
  });

  function getTargetName(type?: TwitchEventSource["type"]) {
    if (type === undefined) {
      return "Select Event";
    }

    switch (type) {
      case "ChannelPoints":
        return "Channel Points redeemed";
      case "BitDonation":
        return "Bit donation received";
      case "Whisper":
        return "Whisper received";
      case "Message":
        return "Message received";
    }
  }

  function getRewardName(reward: string | undefined) {
    if (reward === "" || reward === undefined) return "Any Reward";

    return (
      $customRewardsStore.find((r) => r.id === reward)?.title ??
      "Unknown reward"
    );
  }
</script>

<Card.Root>
  <Card.Content class="grid grid-cols-2 gap-2">
    When
    <Select.Root
      type="single"
      bind:value={
        () => match?.type,
        (newType) => {
          if ((newType as string) === "") return;

          onchange({
            type: newType as TwitchEventSource["type"],
          });
        }
      }
    >
      <Select.Trigger class={placeholder ? "text-muted-foreground" : ""}>
        {getTargetName(match?.type)}
      </Select.Trigger>
      <Select.Content>
        {#each twitchFilterDemoObjects as demo}
          <Select.Item value={demo.type}>{getTargetName(demo.type)}</Select.Item
          >
        {/each}
      </Select.Content>
    </Select.Root>
    {#if match?.type === "ChannelPoints"}
      On
      <Select.Root
        type="single"
        bind:value={
          () => match.reward_id ?? "",
          (newReward) => {
            if (newReward == "") {
              let { reward_id, ...rest } = match;
              onchange(rest);
            } else {
              onchange({
                ...match,
                reward_id: newReward,
              });
            }
          }
        }
      >
        <Select.Trigger>{getRewardName(match.reward_id)}</Select.Trigger>
        <Select.Content>
          <Select.Item value="">Any Reward</Select.Item>
          {#each $customRewardsStore as reward (reward.id)}
            <Select.Item value={reward.id}>{reward.title}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    {:else if match?.type === "BitDonation"}
      Above
      <Input
        type="number"
        min="0"
        value={match.amount ?? ""}
        oninput={(e) => {
          const val = (e.currentTarget as HTMLInputElement).value;
          if (val === "") {
            let { amount, ...rest } = match;
            onchange(rest);
          } else {
            onchange({
              ...match,
              amount: parseInt(val),
            });
          }
        }}
        placeholder="500"
      />
      With text
      <Input
        type="text"
        value={match.message ?? ""}
        oninput={(e) => {
          const val = (e.currentTarget as HTMLInputElement).value;
          if (val === "") {
            let { message, ...rest } = match;
            onchange(rest);
          } else {
            onchange({
              ...match,
              message: val,
            });
          }
        }}
        placeholder="This String"
      />
    {:else if match?.type === "Whisper" || match?.type === "Message"}
      Containing
      <Input
        type="text"
        bind:value={
          () => match.message ?? "",
          (newMessage) => {
            if (newMessage === "") {
              let { message, ...rest } = match;
              onchange(rest);
            } else {
              onchange({
                ...match,
                message: newMessage,
              });
            }
          }
        }
        placeholder="This String"
      />
    {/if}
    {#if !placeholder}
      <Button
        variant="destructive"
        class="col-span-2"
        onclick={() => {
          onchange(undefined);
        }}
      >
        Delete
      </Button>
    {/if}
  </Card.Content>
</Card.Root>
