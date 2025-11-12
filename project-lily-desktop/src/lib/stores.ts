import { writable, type Writable } from "svelte/store";
import type { OscValue, Service, ServiceStatus } from "../bindings";
import type { ConnectResponse } from "../../../project-lily-common/bindings/ConnectResponse";
import { persisted } from "svelte-persisted-store";
import type { TwitchEventSource } from "../../../project-lily-common/bindings/TwitchEventSource";
import type { CustomRewardResponse } from "../../../project-lily-common/bindings/CustomRewardResponse";
import type { StreamLabsEventMatcher } from "./streamlabs";

export const defaultRewardStore: RewardStoreState = {
    rewards: [
        {
            "setsAvatar": "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363",
            "setParams": {},
            "title": "Furry Mode",
            "on": { "type": "twitch", "matches": [{ "type": "ChannelPoints", "reward_id": "f4a6e0a9-72c2-4590-83b8-6c631e6e57c7" }, { "type": "BitDonation", "amount": 500, "message": "!FurryMode" }] }, "timeoutSeconds": 300
        },
        {
            "setsAvatar": "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c",
            "setParams": {
                "new_param_1": "value"
            },
            "title": "Maid Mode (Twitch)",
            "timeoutSeconds": 300,
            "on": {
                "type": "twitch",
                "matches": [
                    {
                        "type": "BitDonation",
                        "amount": 500,
                        "message": "MaidMode"
                    }
                ]
            }
        },
        {
            "setsAvatar": "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c",
            "setParams": {
                "new_param_1": "value"
            },
            "title": "Maid Mode (StreamLabs)",
            "timeoutSeconds": 300,
            "on": {
                "type": "streamlabs",
                "matches": [
                    {
                        "type": "donation",
                        "amount": 5,
                        "message": "MaidMode"
                    }
                ]
            }
        }
    ],
    baseAvatarId: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c",
    baseParams: {}
};

export const oscStateStore: Writable<{ [key: string]: OscValue }> = writable({});
export const serviceStateStore: Writable<Record<Service, ServiceStatus>> = writable({
    "Osc": "Stopped",
    "OBS": "Stopped",
});
export const taskStateStore: Writable<{ [key: string]: { state: TaskState; reason: string; error?: string; } }> = writable({});
export const rewardStore: Writable<RewardStoreState> = persisted(
    "rewardStore",
    defaultRewardStore
);
export const customRewardsStore: Writable<CustomRewardResponse[]> = writable([]);
export const eventLogStore: Writable<TwitchEventSource[]> = writable([]);
export const rewardQueue: Writable<Reward[]> = writable([]);
export const currentReward: Writable<Reward | null> = writable(null);

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

export type Trigger = { type: "twitch" } & { matches: Partial<TwitchEventSource>[] } | { type: "streamlabs" } & { matches: StreamLabsEventMatcher[] };
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