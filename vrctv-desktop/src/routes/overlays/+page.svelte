<script lang="ts">
  import { overlays, overlayVisibleStore } from "$lib/stores";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { OverlayItem } from "../../bindings";
  import { Folder, Plus } from "@lucide/svelte";
  import { readFile } from "@tauri-apps/plugin-fs";
  import * as Card from "$lib/components/ui/card";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Button from "$lib/components/ui/button/button.svelte";
  import OverlayPreview from "$lib/components/overlayPreview.svelte";

  function updateOverlay<T extends keyof OverlayItem>(
    id: number,
    field: T,
    value: OverlayItem[T]
  ) {
    let overlay = $overlays.find((o) => o.id === id);

    if (!overlay) return;

    overlay = {
      ...overlay,
      [field]: value,
    };

    $overlays = $overlays.map((o) => (o.id === id ? overlay! : o));
  }
</script>

<h2 class="font-bold text-xl mb-4">
  Overlays
  <Button
    variant="outline"
    size="icon"
    onclick={() => {
      const newId = $overlays.length
        ? Math.max(...$overlays.map((k) => k.id)) + 1
        : 1;

      const newOverlay: OverlayItem = {
        id: newId,
        name: `Overlay ${newId}`,
        url: "http://example.com",
        visible: false,
        isIframe: false,
      };

      $overlays = [...$overlays, newOverlay];
    }}
  >
    <Plus size="28" />
  </Button>
</h2>
<div class="grid lg:grid-cols-2 xl:grid-cols-3 gap-2">
  {#each $overlays as overlay}
    <Card.Root>
      <Card.Header>
        <div class="grid items-center gap-1.5 mb-4">
          <Label>Name</Label>
          <div class="flex flex-row items-center space-x-2">
            <Input
              placeholder="Name"
              bind:value={
                () => overlay.name,
                (newName) => updateOverlay(overlay.id, "name", newName)
              }
            />
            <Tabs.Root
              bind:value={
                () => (overlay.isIframe ? "iframe" : "img"),
                (newType) =>
                  updateOverlay(overlay.id, "isIframe", newType === "iframe")
              }
            >
              <Tabs.List>
                <Tabs.Trigger value="iframe">Iframe</Tabs.Trigger>
                <Tabs.Trigger value="img">Image</Tabs.Trigger>
              </Tabs.List>
            </Tabs.Root>
          </div>
        </div>
        <div class="grid items-center gap-1.5">
          <Label>Url</Label>
          <div class="flex flex-row items-center space-x-2">
            <Input
              placeholder="Url"
              bind:value={
                () => overlay.url,
                (newUrl) => updateOverlay(overlay.id, "url", newUrl)
              }
            />
            <Button
              variant="outline"
              size="icon"
              onclick={async () => {
                const file = await open({
                  multiple: false,
                  directory: false,
                });

                if (file && typeof file === "string") {
                  // Convert file to base64 URL
                  const fileData = await readFile(
                    `file://${file.replaceAll("\\", "/")}`
                  );
                  const base64Url = `data:text/html;base64,${btoa(
                    new Uint8Array(fileData).reduce(
                      (data, byte) => data + String.fromCharCode(byte),
                      ""
                    )
                  )}`;
                  updateOverlay(overlay.id, "url", base64Url);
                }
              }}
            >
              <Folder />
            </Button>
          </div>
        </div>
      </Card.Header>
      <hr />
      <Card.Content class="flex flex-row space-x-2">
        <OverlayPreview {overlay} class="max-w-lg flex-1" />
        <div>
          <div class="grid items-center gap-1.5 mb-4">
            <Label>By Default</Label>
            <Tabs.Root
              bind:value={
                () => (overlay.visible ? "show" : "hide"),
                (newVisible) =>
                  updateOverlay(overlay.id, "visible", newVisible === "show")
              }
            >
              <Tabs.List>
                <Tabs.Trigger value="show">Show</Tabs.Trigger>
                <Tabs.Trigger value="hide">Hide</Tabs.Trigger>
              </Tabs.List>
            </Tabs.Root>
          </div>
          <div class="grid items-center gap-1.5">
            <Label>Manual Override</Label>
            <Tabs.Root
              bind:value={
                () => ($overlayVisibleStore[overlay.id] ? "show" : "hide"),
                (newVisible) =>
                  ($overlayVisibleStore[overlay.id] = newVisible === "show")
              }
            >
              <Tabs.List>
                <Tabs.Trigger value="show">Show</Tabs.Trigger>
                <Tabs.Trigger value="hide">Hide</Tabs.Trigger>
              </Tabs.List>
            </Tabs.Root>
          </div>
        </div>
      </Card.Content>
      <Card.Footer class="space-x-2 justify-between mt-auto">
        <Button
          class="flex-3"
          onclick={() => {
            const link = `http://localhost:10627/overlay/${overlay.id}`;
            navigator.clipboard.writeText(link);
          }}
        >
          Copy URL for OBS
        </Button>
        <Button
          variant="destructive"
          class="flex-1"
          onclick={() => {
            $overlays = $overlays.filter((o) => o.id !== overlay.id);
          }}
        >
          Delete
        </Button>
      </Card.Footer>
    </Card.Root>
  {/each}
</div>
