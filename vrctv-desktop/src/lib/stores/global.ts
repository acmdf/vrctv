import { writable, type Writable } from "svelte/store";
import * as ENV from "$env/static/public";
import type { OscValue } from "../../bindings";
import type { ConnectResponse } from "../../../../vrctv-common/bindings/ConnectResponse";
import { persisted } from "svelte-persisted-store";

interface LocalState {
    connected: boolean;
    id: string | null;
}

type ClientState = LocalState & ConnectResponse;

export const oscStateStore: Writable<{ [key: string]: OscValue }> = writable({});
export const clientStateStore: Writable<ClientState> = writable({
    connected: false,
    id: null,
    has_twitch: false,
    has_streamlabs: false,
    twitch_id: null,
    twitch_name: null,
    streamlabs_id: null,
    streamlabs_name: null
})

export const wssUrl: Writable<string> = persisted("PUBLIC_WEBSOCKET_URL", "PUBLIC_WEBSOCKET_URL" in ENV ? ENV.PUBLIC_WEBSOCKET_URL as string : "");
export const backendUrl: Writable<string> = persisted("PUBLIC_BACKEND_URL", "PUBLIC_BACKEND_URL" in ENV ? ENV.PUBLIC_BACKEND_URL as string : "");