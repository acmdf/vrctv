<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import { Minus } from "@lucide/svelte";
  import Input from "./ui/input/input.svelte";
  import { getAvatarOscs } from "$lib/avatar-list-cache";

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

  const params = $derived(avatarId ? await getAvatarOscs(avatarId) : undefined);

  $effect(() => {
    if (!avatarId || !params) return;

    if (param && !params.includes(param.replace("/avatar/parameters/", ""))) {
      onChange(null, value || "");
    }
  });
</script>

{#if params && params.length <= 0}
  <p class="text-yellow-500 p-2">No parameters found for this avatar.</p>
{:else}
  <div class="flex flex-row items-center space-x-2">
    {#if params}
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
          {#each removeUselessParams(params) as p}
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
