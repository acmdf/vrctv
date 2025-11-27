<script lang="ts">
    import { overlays, overlayVisibleStore } from "$lib/stores";
    import { open } from "@tauri-apps/plugin-dialog";
    import { type OverlayItem } from "../../bindings";
    import { Minus } from "@lucide/svelte";
    import { readFile } from "@tauri-apps/plugin-fs";

    function updateOverlay<T extends keyof OverlayItem>(
        id: number,
        field: T,
        value: OverlayItem[T],
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

{#each $overlays as overlay}
    <div class="mb-4 p-4 bg-gray-800 rounded">
        <div class="font-bold mb-2">
            Overlay ID: {overlay.id}
            <Minus
                class="inline ml-2 cursor-pointer hover:text-gray-300"
                onclick={() => {
                    $overlays = $overlays.filter((o) => o.id !== overlay.id);
                }}
            />
            <button
                class="ml-2 px-2 py-1 bg-gray-700 rounded hover:bg-gray-600 cursor-pointer"
                onclick={() => {
                    const link = `http://localhost:10627/overlay/${overlay.id}`;
                    navigator.clipboard.writeText(link);
                }}>Copy URL</button
            >
        </div>
        <div class="mb-2">
            Name: <input
                value={overlay.name}
                placeholder="Name"
                oninput={(e) =>
                    updateOverlay(overlay.id, "name", e.currentTarget.value)}
            />
        </div>
        <div class="mb-2">
            URL: <input
                value={overlay.url}
                placeholder="URL"
                oninput={(e) =>
                    updateOverlay(overlay.id, "url", e.currentTarget.value)}
            />
            <button
                class="ml-2 px-2 py-1 bg-grey-700 rounded hover:bg-grey-600 cursor-pointer"
                onclick={async () => {
                    const file = await open({
                        multiple: false,
                        directory: false,
                    });

                    if (file && typeof file === "string") {
                        // Convert file to base64 URL
                        const fileData = await readFile(
                            `file://${file.replaceAll("\\", "/")}`,
                        );
                        const base64Url = `data:text/html;base64,${btoa(
                            new Uint8Array(fileData).reduce(
                                (data, byte) =>
                                    data + String.fromCharCode(byte),
                                "",
                            ),
                        )}`;
                        updateOverlay(overlay.id, "url", base64Url);
                    }
                }}>Browse</button
            >
        </div>
        <!-- Visible by default -->
        <div class="mb-2">
            Visible by default:
            <input
                type="checkbox"
                checked={overlay.visible}
                onchange={(e) =>
                    updateOverlay(
                        overlay.id,
                        "visible",
                        e.currentTarget.checked,
                    )}
            />
        </div>
        <!-- Is Iframe -->
        <div class="mb-2">
            Is Iframe:
            <input
                type="checkbox"
                checked={overlay.isIframe}
                onchange={(e) =>
                    updateOverlay(
                        overlay.id,
                        "isIframe",
                        e.currentTarget.checked,
                    )}
            />
        </div>
        <div class="flex space-x-2">
            <button
                class="px-4 py-2 bg-green-600 rounded hover:bg-green-700 cursor-pointer"
                onclick={() => {
                    $overlayVisibleStore[overlay.id] = true;
                }}
            >
                Show
            </button>
            <button
                class="px-4 py-2 bg-red-600 rounded hover:bg-red-700 cursor-pointer"
                onclick={() => {
                    $overlayVisibleStore[overlay.id] = false;
                }}
            >
                Hide
            </button>
        </div>
    </div>
{/each}
<button
    class="mt-4 px-4 py-2 bg-blue-600 rounded hover:bg-blue-700"
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
    Add New Overlay
</button>
