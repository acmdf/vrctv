<script lang="ts">
  import { commands } from "../../bindings";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import { Minus } from "@lucide/svelte";
  import Input from "./ui/input/input.svelte";

  const {
    avatarId,
    param,
    value,
    onChange,
    placeholder = "",
  }: {
    avatarId?: string;
    param: string | null;
    value?: string;
    onChange: (param: string | null, value: string) => void;
    placeholder?: string;
  } = $props();

  function removeUselessParams(params: string[]): string[] {
    return params.filter(
      (p) =>
        !p.includes("SyncData") &&
        ["FT/v2", "Go/"].every(
          (uselessPrefix) => !p.startsWith(uselessPrefix),
        ) &&
        [
          "/LastSynced",
          "_Squish",
          "_Stretch",
          "_Angle",
          "_IsPosed",
          "_IsGrabbed",
        ].every((uselessEnding) => !p.endsWith(uselessEnding)),
    );
  }

  const triggerText = $derived(
    param && param.startsWith("/avatar/parameters/")
      ? "Set " + param.replace("/avatar/parameters/", "")
      : placeholder,
  );

  const paramsLoad = $derived(
    avatarId ? await commands.fetchAvatarOsc(avatarId) : undefined,
  );

  $effect(() => {
    if (!avatarId || !paramsLoad) return;

    if (
      param &&
      paramsLoad.status === "ok" &&
      !paramsLoad.data.includes(param.replace("/avatar/parameters/", ""))
    ) {
      onChange(null, value || "");
    }
  });
</script>

{#if paramsLoad && paramsLoad.status !== "ok"}
  <p class="text-red-500 p-2">Error: {paramsLoad.error}</p>
{:else if paramsLoad && paramsLoad.data.length <= 0}
  <p class="text-yellow-500 p-2">No parameters found for this avatar.</p>
{:else}
  <div class="flex flex-row items-center p-2 space-x-2">
    {#if paramsLoad}
      <Select.Root
        type="single"
        bind:value={
          () => param || "",
          (selectedParam: string) => {
            onChange(selectedParam, value || "");
          }
        }
      >
        <Select.Trigger
          class={placeholder
            ? "text-muted-foreground xl:min-w-sm"
            : "xl:min-w-sm"}
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
    {:else}
      <Input
        bind:value={
          () => param || "",
          (selectedParam: string) => {
            onChange(selectedParam, value || "");
          }
        }
        class="xl:min-w-sm"
        placeholder={triggerText}
      />
    {/if}
    {#if value !== undefined}
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
    {/if}
    {#if !placeholder}
      <Minus
        class="text-red-500 cursor-pointer h-4"
        size="64"
        strokeWidth={8}
        onclick={() => {
          onChange(null, value || "");
        }}
      />
    {/if}
  </div>
{/if}
