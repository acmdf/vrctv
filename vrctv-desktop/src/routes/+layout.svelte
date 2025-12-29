<script lang="ts">
    // Tailwind styling
    import "../app.css";

    import toast, { Toaster } from "svelte-french-toast";
    import { commands, events } from "../bindings";
    import { onMount } from "svelte";
    import Menu from "./menu.svelte";
    import Logo from "$lib/components/logo.svelte";
    import StoreManager from "$lib/components/storeManager.svelte";
    import ConnectionManager from "$lib/components/connectionManager.svelte";
    import { attachConsole, info } from "@tauri-apps/plugin-log";
    import { overlays, overlayVisibleStore } from "$lib/stores";
    import { currentOverlayState } from "$lib/overlays";
    import { ModeWatcher } from "mode-watcher";
    import ThemeSwitcher from "$lib/components/themeSwitcher.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";

    onMount(async () => {
        await attachConsole();
    });
    const { children } = $props();

    onMount(() => {
        events.serviceStatusEvent.listen((event) => {
            if (event.payload.status === "Started") {
                toast.success(`${event.payload.service} service started`);
            } else if (event.payload.status === "Stopped") {
                toast.error(`${event.payload.service} service stopped`);
            } else if (Object.hasOwn(event.payload.status, "Error")) {
                toast.error(
                    `${event.payload.service} service error: ${event.payload.status.Error}`,
                );
            }
        });
    });

    // Subscribe to the overlays store
    $effect(() => {
        info(
            "Overlays updated: " +
                JSON.stringify($overlays, (k, v) => {
                    if (k === "url") {
                        return v.slice(0, 20) + "...";
                    }
                    return v;
                }),
        );
        commands.updateOverlays(
            currentOverlayState($overlayVisibleStore, $overlays),
        );
    });
</script>

<Sidebar.Provider>
    <Menu />
    <main class="w-full">
        <!-- Centered Logo -->
        <header class="bg-sidebar text-sidebar-foreground p-4 relative">
            <Sidebar.Trigger class="absolute left-4 top-5" />
            <div class="flex items-center justify-center">
                <Logo cogSpinning={false} width="40" height="40" />
            </div>
        </header>
        <div class="flex-grow p-4">
            {@render children()}
        </div>
    </main>
</Sidebar.Provider>

<div class="fixed bottom-4 right-4 z-50">
    <ThemeSwitcher />
</div>

<!-- Services -->
<Toaster position="bottom-center" />
<StoreManager />
<ConnectionManager />
<ModeWatcher />
