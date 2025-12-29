<script lang="ts">
  import Label from "$lib/components/ui/label/label.svelte";
  import * as Select from "$lib/components/ui/select";
  import type { Avatar } from "../../bindings";

  let {
    label = "",
    avatars,
    avatarId = $bindable(undefined),
  }: {
    label: string;
    avatars: Avatar[];
    avatarId?: string;
  } = $props();

  function lookupAvatarText(avatarId: string | undefined): string {
    return (
      avatars.find((a) => a.id === avatarId)?.name ?? "Please select an avatar"
    );
  }
</script>

<div class="grid items-center gap-1.5">
  <Label>{label}</Label>
  <div class="flex flex-row items-center mb-4">
    <Select.Root bind:value={avatarId} type="single">
      <Select.Trigger>
        {lookupAvatarText(avatarId)}
      </Select.Trigger>
      <Select.Content align="start">
        {#each avatars as avatar}
          <Select.Item value={avatar.id}>
            {avatar.name} ({avatar.id})
          </Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
    <div class="ml-4 text-muted-foreground text-xs">
      {avatarId}
    </div>
  </div>
</div>
