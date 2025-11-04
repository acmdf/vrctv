import type { ClientMessage } from "../../../project-lily-common/bindings/ClientMessage";
import type { ServerMessage } from "../../../project-lily-common/bindings/ServerMessage";
import { clientStateStore, customRewardsStore, eventLogStore, TaskState, taskStateStore } from "./stores";
import toast from "svelte-french-toast";
import { debug, error, info } from "@tauri-apps/plugin-log";
import { commands } from "../bindings";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { writable } from "svelte/store";
import { handleTwitchEvent } from "./twitch";
import type WebSocket from "@tauri-apps/plugin-websocket";
import type { MessageKind } from "@tauri-apps/plugin-websocket";

export const serverConnection = writable<ServerConnection | null>(null);

class ServerConnection {
    private websocket: WebSocket;
    private requestNo = 0;
    private intervalHandle: number | null = null;
    public connected = false;

    constructor(websocket: WebSocket, retryMethod: () => void) {
        this.websocket = websocket;

        // Start pinging every 30 seconds
        this.intervalHandle = setInterval(async () => {
            if (this.connected) {
                try {
                    await this.websocket?.send({ type: "Ping", data: [this.requestNo] });
                } catch (e) {
                    error(`Failed to send ping: ${e}`);
                    this.close();
                    retryMethod();
                }
            }
        }, 30000);

        window.onbeforeunload = () => {
            this.close();
        };
    }

    async close() {
        if (this.intervalHandle) {
            clearInterval(this.intervalHandle);
            this.intervalHandle = null;
        }
        try {
            await this.websocket.disconnect();
        } catch (e) {
            error(`Failed to disconnect websocket: ${e}`);
        }
        this.connected = false;
    }

    send(data: ClientMessage) {
        try {
            this.websocket.send(JSON.stringify(data));
        } catch (e) {
            error(`Failed to send message: ${e}`);
            this.close();
        }
    }

    getNextRequestId(reason: string): number {
        debug(`Getting next request ID for: ${reason}`);
        taskStateStore.update(state => ({
            ...state,
            [this.requestNo]: {
                state: TaskState.InProgress,
                reason: reason
            }
        }));

        return this.requestNo++;
    }

    static parse_message(message: MessageKind<"Text", string>): ServerMessage | null {
        try {
            const data = JSON.parse(message.data);
            return data as ServerMessage;
        } catch (e) {
            console.error("Failed to parse message", e);
        }
        return null;
    }
}

export function onConnect(i: WebSocket, retryMethod: () => void): ServerConnection {
    let conn = new ServerConnection(i, retryMethod);
    serverConnection.set(conn);
    conn.connected = true;
    let stateToken = localStorage.getItem("stateToken");
    info(`WebSocket connection established with state token: ${stateToken}`);

    if (stateToken) {
        clientStateStore.update(state => ({ ...state, id: stateToken }));
        conn.send({ type: "connect", state_token: stateToken });
    } else {
        conn.send({ type: "codeRequest" } as ClientMessage);
    }

    return conn;
}

export async function sendNotif(title: string, message: string) {
    info(`Sending notification: ${title} - ${message}`);
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
    }
    if (permissionGranted) {
        await sendNotification({ title, body: message });
    }
    commands.sendNotification({
        messageType: 1,
        index: 0,
        timeout: 5,
        height: 175,
        opacity: 1,
        volume: 1,
        audioPath: "default",
        title,
        content: message,
        useBase64Icon: false,
        icon: "default",
        sourceApp: "Project Lily"
    });
}


export function handleMessage(message: MessageKind<"Text", string>) {
    let parsed = ServerConnection.parse_message(message);

    if (!parsed) return;

    debug(`Received message ${JSON.stringify(parsed)}`);

    switch (parsed.type) {
        case "codeResponse":
            if (parsed.state_token) {
                localStorage.setItem("stateToken", parsed.state_token);
                clientStateStore.update(state => ({ ...state, id: parsed.state_token }));

                break;
            }
            toast.error("Could not get the connection token from the server.");
            break;
        case "connectResponse":
            // Remove type field
            let { type, ...rest } = parsed;

            // Merge the rest of the fields into the client state store
            clientStateStore.update(state => ({ ...state, ...rest }));
            break;
        case "changeAvatar":
            commands.changeAvatar(parsed.id);
            info(`Changing avatar to ${parsed.id}`);
            toast.success(`Avatar changed to ${parsed.id}`);
            break;
        case "error":
            toast.error(`Error from server: ${parsed.message}`);
            error(`Error from server: ${parsed.message}`);
            if (parsed.request_id) {
                taskStateStore.update(state => ({
                    ...state,
                    [parsed.request_id]: {
                        state: TaskState.Failed,
                        reason: state[parsed.request_id]?.reason || "Unknown",
                        error: parsed.message || "Unknown error"
                    }
                }));
            }
            break;
        case "notify":
            sendNotif(parsed.title, parsed.message);
            break;
        case "taskResponse":
            taskStateStore.update(state => ({
                ...state,
                [parsed.request_id]: {
                    state: parsed.success ? TaskState.Completed : TaskState.Failed,
                    reason: state[parsed.request_id]?.reason || "Unknown",
                    error: parsed.success ? undefined : (parsed.message || "Unknown error")
                }
            }));
            info(`Task ${parsed.request_id} completed with success: ${parsed.success}`);
            if (parsed.message) {
                toast[
                    parsed.success ? "success" : "error"
                ](parsed.success ? parsed.message : `Error: ${parsed.message}`);
                error(`Task ${parsed.request_id} completed with message: ${parsed.message}`);
            }
            break;
        case "customRewards":
            info(`Received custom rewards: ${JSON.stringify(parsed.rewards)}`);
            customRewardsStore.set(parsed.rewards);
            break;
        case "twitchEvent":
            info(`Received Twitch event: ${JSON.stringify(parsed)}`);
            // toast.success(`Twitch event: ${JSON.stringify(parsed.event)}`);
            eventLogStore.update(logs => ([...logs, ...(parsed.type === "twitchEvent" ? [parsed.event] : [])]));
            handleTwitchEvent(parsed.event);
            break;
        case "streamLabsEvent":
            info(`Received StreamLabs event: ${JSON.stringify(parsed)}`);
            // toast.success(`StreamLabs event: ${JSON.stringify(parsed.event_key)}`);
            break;
    }
}