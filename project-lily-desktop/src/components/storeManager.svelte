<script lang="ts">
    import { events } from "../bindings";
    import { onMount } from "svelte";
    import { oscStateStore, serviceStateStore } from "../lib/stores";
    import { info, debug } from "@tauri-apps/plugin-log";

    onMount(() => {
        events.oscChangeEvent.listen((event) => {
            debug(`Received OSC Change Event: ${JSON.stringify(event)}`);
            // You can update your state or perform actions based on the event here

            const { address, value } = event.payload;

            oscStateStore.update((state) => ({ ...state, [address]: value }));
        });

        events.serviceStatusEvent.listen((event) => {
            info(`Received Service Status Event: ${JSON.stringify(event)}`);
            const { service, status } = event.payload;

            serviceStateStore.update((state) => ({
                ...state,
                [service]: status,
            }));
        });
    });
</script>
