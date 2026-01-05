import { get, writable, type Writable } from "svelte/store";
import { persisted } from "svelte-persisted-store";
import { SetAvatarReward } from "$lib/rewards/set-avatar";
import { StreamlabsDonationTrigger } from "$lib/triggers/streamlabs-donation";
import { TwitchBitDonationTrigger } from "$lib/triggers/twitch-bit-donation";
import { OrTrigger } from "$lib/triggers/or";
import { TwitchChannelPointsTrigger } from "$lib/triggers/twitch-channel-points";
import { parse, stringify } from "devalue";
import { restoreReward, restoreTrigger } from "$lib/task-parts";
import { TriggerInstance } from "$lib/triggers/types";
import { RewardInstance, type RewardContext } from "$lib/rewards/types";
import type { CustomRewardResponse } from "../../../../vrctv-common/bindings/CustomRewardResponse";
import { RewardHandler } from "$lib/rewardHandler";

export interface Task {
    id: string;
    name: string;
    trigger: TriggerInstance<any>;
    rewards: RewardInstance<any>[];
}

export interface RewardStoreState {
    baseAvatarId: string | undefined;
    tasks: Task[];
}

export const defaultRewardStore: RewardStoreState = {
    tasks: [
        {
            id: crypto.randomUUID(),
            name: "Furry Mode",
            trigger: new OrTrigger({
                subtriggers: [
                    new TwitchChannelPointsTrigger({
                        reward_id: "f4a6e0a9-72c2-4590-83b8-6c631e6e57c7"
                    }),
                    new TwitchBitDonationTrigger({
                        minimum_amount: 500,
                        message_contains: "!FurryMode"
                    }),
                ]
            }),
            rewards: [
                new SetAvatarReward({
                    avatar_id: "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363",
                    return_to: "previous",
                    timeout_ms: 300000
                })
            ]
        },
        {
            id: crypto.randomUUID(),
            name: "Maid Mode",
            trigger: new OrTrigger({
                subtriggers: [
                    new TwitchBitDonationTrigger({
                        minimum_amount: 500,
                        message_contains: "MaidMode"
                    }),
                    new StreamlabsDonationTrigger({
                        minimum_amount: 5,
                        message_contains: "MaidMode"
                    }),
                ]
            }),
            rewards: [
                new SetAvatarReward({
                    avatar_id: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c",
                    return_to: "previous",
                    timeout_ms: 300000
                })
            ]
        }
    ],
    baseAvatarId: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c",
};

export const rewardStore: Writable<RewardStoreState> = persisted(
    "rewardStore",
    defaultRewardStore,
    {
        serializer: {
            parse: (s: string) => {
                return parse(s, {
                    TriggerInstance: restoreTrigger,
                    RewardInstance: restoreReward
                });
            },
            stringify: (obj: RewardStoreState) => {
                return stringify(obj, {
                    TriggerInstance: (t: TriggerInstance<any>) => t instanceof TriggerInstance && t.getStoredTrigger(),
                    RewardInstance: (r: RewardInstance<any>) => r instanceof RewardInstance && r.getStoredReward(),
                });
            }
        }
    }
);
export const customRewardsStore: Writable<CustomRewardResponse[]> = writable([]);
export const rewardHandler: Writable<RewardHandler> = writable(new RewardHandler());

export function updateContext(context: RewardContext): RewardContext {
    return {
        ...context,
        runningRewards: get(rewardHandler).activeRewards.map((ar) => ar[0]),
        rewardQueue: get(rewardHandler).rewardQueue.map((rq) => rq[0]),
        global_values: get(rewardHandler).globalKV,
    };
}