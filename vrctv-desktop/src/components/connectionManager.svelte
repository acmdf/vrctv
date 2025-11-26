<script lang="ts">
    import { onMount } from "svelte";
    import { clientStateStore } from "../lib/stores";
    import { handleMessage, onConnect } from "$lib/websocket";
    import { PUBLIC_WEBSOCKET_URL } from "$env/static/public";
    import { debug, info, warn } from "@tauri-apps/plugin-log";
    import WebSocket from "@tauri-apps/plugin-websocket";

    function restartWebsocket() {
        setTimeout(() => tryCreateWebSocket(), 5000);
    }

    async function tryCreateWebSocket() {
        warn("Attempting to create WebSocket connection...");

        let ws: WebSocket;
        try {
            ws = await WebSocket.connect(PUBLIC_WEBSOCKET_URL);
        } catch (e) {
            clientStateStore.update((s) => ({ ...s, connected: false }));
            info(`WebSocket connection failed: ${e}`);
            restartWebsocket();
            return;
        }
        clientStateStore.update((s) => ({ ...s, connected: true }));
        info(`WebSocket created: ${JSON.stringify(ws)}`);
        const conn = onConnect(ws, restartWebsocket);

        let cancel = ws.addListener((message) => {
            switch (message.type) {
                case "Close":
                    clientStateStore.update((s) => ({
                        ...s,
                        connected: false,
                    }));
                    warn("WebSocket connection closed");
                    conn?.close();
                    cancel();
                    restartWebsocket();
                    break;
                case "Ping":
                    debug("WebSocket ping received");
                    ws.send({ type: "Pong", data: message.data });
                    break;
                case "Pong":
                    debug("WebSocket pong received");
                    break;
                case "Binary":
                    info("WebSocket binary message received: " + message.data);
                    break;
                case "Text":
                    debug(`Websocket text message received: ${message.data}`);
                    handleMessage(message);
                    break;
            }
        });

        return;
    }

    onMount(() => {
        console.warn(
            "Mounting connection manager and trying to create WebSocket",
        );
        tryCreateWebSocket();
    });
</script>
