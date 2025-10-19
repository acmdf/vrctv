<script lang="ts">
    // Tailwind styling
    import "../app.css";

    import toast, { Toaster } from "svelte-french-toast";
    import { events } from "../bindings";
    import { onMount } from "svelte";
    import Menu from "./menu.svelte";
    import Logo from "../components/logo.svelte";
    import StoreManager from "../components/storeManager.svelte";
    import ConnectionManager from "../components/connectionManager.svelte";
    import { attachConsole } from "@tauri-apps/plugin-log";

    onMount(async () => {
        await attachConsole();
    });
    let { children } = $props();

    onMount(() => {
        events.serviceStatusEvent.listen((event) => {
            if (event.payload.status === "Started") {
                toast.success(`${event.payload.service} service started`);
            } else if (event.payload.status === "Stopped") {
                toast.error(`${event.payload.service} service stopped`);
            } else if (event.payload.status.hasOwnProperty("Error")) {
                toast.error(
                    `${event.payload.service} service error: ${event.payload.status.Error}`,
                );
            }
        });
    });
</script>

<div class="flex items-stretch">
    <Menu />
    <div class="h-screen overflow-auto flex flex-col flex-grow">
        <header class="bg-gray-800 p-4 flex space-x-4">
            <Logo cogSpinning={false} width="40" height="40" />
        </header>
        <main class="flex-grow p-4">
            {@render children()}
        </main>
    </div>
</div>
<!-- Services -->
<Toaster position="bottom-center" />
<StoreManager />
<ConnectionManager />
