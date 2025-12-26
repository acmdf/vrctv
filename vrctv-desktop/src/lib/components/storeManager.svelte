<script lang="ts">
    import { events } from "../../bindings";
    import { onMount } from "svelte";
    import { oscStateStore, serviceStateStore } from "../stores";
    import { info } from "@tauri-apps/plugin-log";

    const batchInterval = 100; // milliseconds
    let oscUpdateQueue: Record<string, any> = {};
    let oscBatchTimeout: ReturnType<typeof setInterval> | null = null;

    onMount(() => {
        oscBatchTimeout = setInterval(() => {
            oscStateStore.update((s) => {
                Object.keys(oscUpdateQueue).forEach((key) => {
                    s[key] = oscUpdateQueue[key];
                });
                oscUpdateQueue = {};
                return s;
            });
        }, batchInterval);

        events.oscChangeEvent.listen((event) => {
            // debug(`Received OSC Change Event: ${JSON.stringify(event)}`);
            // You can update your state or perform actions based on the event here

            const { address, value } = event.payload;

            // There are about 600 updates per second, so we batch them before updating the store
            oscUpdateQueue[address] = value;
        });

        events.serviceStatusEvent.listen((event) => {
            info(`Received Service Status Event: ${JSON.stringify(event)}`);
            const { service, status } = event.payload;

            serviceStateStore.update((state) => ({
                ...state,
                [service]: status,
            }));
        });

        return () => {
            if (oscBatchTimeout) {
                clearInterval(oscBatchTimeout);
            }
        };
    });
</script>
