<script lang="ts">
  import { commands } from "../../bindings";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import { Minus } from "@lucide/svelte";

  const {
    avatarId,
    param,
    value,
    onChange,
    placeholder = false,
  }: {
    avatarId: string;
    param: string | null;
    value: string;
    onChange: (param: string | null, value: string) => void;
    placeholder?: boolean;
  } = $props();

  function removeUselessParams(params: string[]): string[] {
    return params.filter(
      (p) =>
        !p.includes("SyncData") &&
        ["FT/v2", "Go/"].every(
          (uselessPrefix) => !p.startsWith(uselessPrefix)
        ) &&
        [
          "/LastSynced",
          "_Squish",
          "_Stretch",
          "_Angle",
          "_IsPosed",
          "_IsGrabbed",
        ].every((uselessEnding) => !p.endsWith(uselessEnding))
    );
  }

  const triggerText = $derived(
    param && param.startsWith("/avatar/parameters/")
      ? "Set " + param.replace("/avatar/parameters/", "")
      : "Set this parameter"
  );

  const paramsLoad = $derived(await commands.fetchAvatarOsc(avatarId));

  $effect(() => {
    const availableParams = paramsLoad.status === "ok" ? paramsLoad.data : [];

    if (
      param &&
      paramsLoad.status === "ok" &&
      !availableParams.includes(param.replace("/avatar/parameters/", ""))
    ) {
      onChange(null, value);
    }
  });
</script>

{#if paramsLoad.status === "ok"}
  {#if paramsLoad.data.length > 0}
    <div class="flex flex-row items-center p-2 space-x-2">
      <Select.Root
        type="single"
        bind:value={
          () => param || "",
          (selectedParam: string) => {
            onChange(selectedParam, value);
          }
        }
      >
        <Select.Trigger
          class={placeholder ? "text-muted-foreground min-w-sm" : "min-w-sm"}
        >
          {triggerText}
        </Select.Trigger>
        <Select.Content>
          {#each removeUselessParams(paramsLoad.data) as p}
            <Select.Item value={`/avatar/parameters/${p}`}>
              {p}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
      <InputGroup.Root>
        <InputGroup.Input
          type="text"
          {value}
          class="ml-2"
          onchange={(e) => {
            onChange(param, (e.currentTarget as HTMLInputElement).value);
          }}
        />
        <InputGroup.Addon>
          <InputGroup.Text>To:</InputGroup.Text>
        </InputGroup.Addon>
      </InputGroup.Root>
      {#if !placeholder}
        <Minus
          class="text-red-500 cursor-pointer h-4"
          size="64"
          strokeWidth={8}
          onclick={() => {
            onChange(null, value);
          }}
        />
      {/if}
    </div>
  {:else}
    <p class="text-yellow-500 p-2">No parameters found for this avatar.</p>
  {/if}
{:else}
  <p class="text-red-500 p-2">Error: {paramsLoad.error}</p>
{/if}
