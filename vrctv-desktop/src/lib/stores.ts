import { writable, type Writable } from "svelte/store";
import type { OscValue, OverlayItem, Service, ServiceStatus } from "../bindings";
import type { ConnectResponse } from "../../../vrctv-common/bindings/ConnectResponse";
import { persisted } from "svelte-persisted-store";
import type { TwitchEventSource } from "../../../vrctv-common/bindings/TwitchEventSource";
import type { CustomRewardResponse } from "../../../vrctv-common/bindings/CustomRewardResponse";
import type { StreamLabsEventMatcher } from "./streamlabs";

export const defaultRewardStore: RewardStoreState = {
    rewards: [
        {
            type: "avatar",
            "setsAvatar": "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363",
            "setParams": {},
            "title": "Furry Mode",
            "on": { "type": "twitch", "matches": [{ "type": "ChannelPoints", "reward_id": "f4a6e0a9-72c2-4590-83b8-6c631e6e57c7" }, { "type": "BitDonation", "amount": 500, "message": "!FurryMode" }] }, "timeoutSeconds": 300
        },
        {
            type: "avatar",
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
            type: "avatar",
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
    "Overlay": "Stopped",
});
export const taskStateStore: Writable<{ [key: string]: { state: TaskState; reason: string; error?: string; } }> = writable({});
export const rewardStore: Writable<RewardStoreState> = persisted(
    "rewardStore",
    defaultRewardStore
);
export const customRewardsStore: Writable<CustomRewardResponse[]> = writable([]);
export const eventLogStore: Writable<TwitchEventSource[]> = writable([]);
export const avatarRewardQueue: Writable<AvatarReward[]> = writable([]);
export const currentAvatarRewardTimeout: Writable<number | null> = writable(null);
export const overlayRewardQueue: Writable<{ [key: number]: OverlayReward[] }> = writable([]);
export const currentOverlayRewardTimeout: Writable<{ [key: number]: number | null }> = writable({});
export const currentReward: Writable<Reward | null> = writable(null);
export const overlays: Writable<OverlayItem[]> = persisted(
    "overlaysStore",
    []
);
export const overlayVisibleStore: Writable<Record<number, boolean>> = writable({});

export type Reward = AvatarReward | AvatarCancelReward | OverlayReward | OverlayCancelReward;

export interface RewardStoreState {
    baseAvatarId: string | undefined;
    baseParams: Record<string, string>;
    rewards: Reward[];
}

export interface AvatarReward {
    setsAvatar: string | undefined;
    setParams: Record<string, string>;
    title: string;
    timeoutSeconds: number;
    on: Trigger;
    type: "avatar";
}

export interface AvatarCancelReward {
    title: string;
    on: Trigger;
    type: "avatarCancel";
}

export interface OverlayReward {
    overlay: number;
    show: boolean;
    title: string;
    timeoutSeconds: number;
    on: Trigger;
    type: "overlay";
}

export interface OverlayCancelReward {
    overlay: number;
    title: string;
    on: Trigger;
    type: "overlayCancel";
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