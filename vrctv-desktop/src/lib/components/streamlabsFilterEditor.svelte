<script lang="ts">
  import type { StreamLabsEventMatcher } from "$lib/streamlabs";
  import * as Card from "$lib/components/ui/card";
  import * as InputGroup from "$lib/components/ui/input-group";
  import * as Select from "$lib/components/ui/select";
  import Input from "./ui/input/input.svelte";
  import Button from "./ui/button/button.svelte";

  const {
    placeholder = false,
    match,
    onchange,
  }: {
    placeholder?: boolean;
    match?: StreamLabsEventMatcher;
    onchange: (newMatch?: StreamLabsEventMatcher) => void;
  } = $props();

  const streamlabsFilterDemoObjects: StreamLabsEventMatcher[] = [
    {
      type: "donation",
      amount: 500,
      message: "MaidMode",
    },
  ];

  function getTargetName(type?: StreamLabsEventMatcher["type"]) {
    if (type === undefined) {
      return "Select Event";
    }

    switch (type) {
      case "donation":
        return "Donation Received";
    }
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
            type: newType as StreamLabsEventMatcher["type"],
          });
        }
      }
    >
      <Select.Trigger class={placeholder ? "text-muted-foreground" : ""}>
        {getTargetName(match?.type)}
      </Select.Trigger>
      <Select.Content>
        {#each streamlabsFilterDemoObjects as demo}
          <Select.Item value={demo.type}>{getTargetName(demo.type)}</Select.Item
          >
        {/each}
      </Select.Content>
    </Select.Root>

    {#if match?.type === "donation"}
      Above
      <InputGroup.Root>
      <InputGroup.Input
        type="number"
        min="0"
        bind:value={
          () => match.amount ?? 0,
          (newAmount) => {
            if (newAmount === 0) {
              let { amount, ...rest } = match;
              onchange(rest);
            } else {
              onchange({
                ...match,
                amount: newAmount,
              });
            }
          }
        }
        placeholder="Minimum Amount"
      />
      <InputGroup.Addon align="inline-end"><InputGroup.Text>In Local Currency</InputGroup.Text></InputGroup.Addon>
      </InputGroup.Root>
      Containing Message
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
