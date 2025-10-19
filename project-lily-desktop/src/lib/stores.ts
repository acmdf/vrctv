import { writable } from "svelte/store";
import type { OscValue, Service, ServiceStatus } from "../bindings";
import type { ConnectResponse } from "../../../project-lily-common/bindings/ConnectResponse";
import { persisted } from "svelte-persisted-store";
import type { TwitchEventSource } from "../../../project-lily-common/bindings/TwitchEventSource";
import type { StreamLabsEvent } from "../../../project-lily-common/bindings/StreamLabsEvent";
import type { CustomRewardResponse } from "../../../project-lily-common/bindings/CustomRewardResponse";

export const oscStateStore = writable<{ [key: string]: OscValue }>({});
export const serviceStateStore = writable<Record<Service, ServiceStatus>>({
    "Osc": "Stopped"
});
export const taskStateStore = writable<{ [key: string]: { state: TaskState; reason: string } }>({});
export const rewardStore = persisted(
    "rewardStore",
    { rewards: [{ "setsAvatar": "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363", "setParams": {}, "title": "Furry Mode", "on": { "type": "twitch", "matches": [{ "type": "ChannelPoints", "reward_id": "f4a6e0a9-72c2-4590-83b8-6c631e6e57c7" }, { "type": "BitDonation", "amount": 500, "message": "!FurryMode" }] }, "timeoutSeconds": 300 }], baseAvatarId: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c", baseParams: {} } as RewardStoreState,
);
export const customRewardsStore = writable<CustomRewardResponse[]>([]);
export const eventLogStore = writable<TwitchEventSource[]>([]);
export const rewardQueue = writable<Reward[]>([]);
export const currentReward = writable<Reward | null>(null);

export interface RewardStoreState {
    baseAvatarId: string | null;
    baseParams: Record<string, string>;
    rewards: Reward[];
}

export interface Reward {
    setsAvatar: string | null;
    setParams: Record<string, string>;
    title: string;
    timeoutSeconds: number;
    on: Trigger;
}

export type Trigger = { type: "twitch" } & { matches: Partial<TwitchEventSource>[] } | { type: "streamlabs" } & { matches: Partial<StreamLabsEvent>[] };
export const twitchEventSources: (TwitchEventSource["type"])[] = [
    "ChannelPoints",
    "BitDonation",
    "Whisper",
    "Message"
];

export enum TaskState {
    InProgress = "InProgress",
    Completed = "Completed",
    Failed = "Failed"

}

interface LocalState {
    connected: boolean;
    id: string | null;
}

type ClientState = LocalState & ConnectResponse;

export const clientStateStore = writable<ClientState>({
    connected: false,
    id: null,
    has_twitch: false,
    has_streamlabs: false,
    twitch_id: null,
    twitch_name: null,
    streamlabs_id: null,
    streamlabs_name: null
})