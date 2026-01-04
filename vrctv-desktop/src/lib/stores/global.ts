import { writable, type Writable } from "svelte/store";
import type { OscValue } from "../../bindings";
import type { ConnectResponse } from "../../../../vrctv-common/bindings/ConnectResponse";

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