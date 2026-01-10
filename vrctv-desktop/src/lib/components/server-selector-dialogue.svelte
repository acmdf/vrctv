<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { buttonVariants } from "./ui/button";
  import { wssUrl, backendUrl } from "$lib/stores/global";
  import Input from "./ui/input/input.svelte";
  import Label from "./ui/label/label.svelte";

  let { open = $bindable() } = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>
        Please enter the server urls you want to connect to
      </Dialog.Title>
      <div class="grid flex-1 gap-2">
        <Label for="backend">Backend Url (Used for TTV/Streamlabs Auth)</Label>
        <Input
          id="backend"
          bind:value={$backendUrl}
          placeholder="https://example.com/"
        />
      </div>
      <div class="grid flex-1 gap-2">
        <Label for="wss">WSS Url (Used for monitoring events)</Label>
        <Input
          id="wss"
          bind:value={$wssUrl}
          placeholder="wss://example.com/ws"
        />
      </div>
      <Dialog.Footer class="sm:justify-start">
        <Dialog.Close class={buttonVariants({ variant: "secondary" })}
          >Set</Dialog.Close
        >
      </Dialog.Footer>
    </Dialog.Header>
  </Dialog.Content>
</Dialog.Root>
