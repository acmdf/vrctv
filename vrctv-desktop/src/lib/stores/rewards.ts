import { get, writable, type Writable } from "svelte/store";
import { persisted } from "svelte-persisted-store";
import { SetAvatarReward } from "$lib/rewards/set-avatar";
import { StreamlabsDonationTrigger } from "$lib/triggers/streamlabs-donation";
import { TwitchBitDonationTrigger } from "$lib/triggers/twitch-bit-donation";
import { OrTrigger } from "$lib/triggers/or";
import { parse, stringify } from "devalue";
import { restoreReward, restoreTrigger } from "$lib/task-parts";
import { TriggerInstance } from "$lib/triggers/types";
import { RewardInstance, type RewardContext } from "$lib/rewards/types";
import type { CustomRewardResponse } from "../../../../vrctv-common/bindings/CustomRewardResponse";
import { RewardHandler } from "$lib/reward-handler";
import { TwitchMessageTrigger } from "$lib/triggers/twitch-message";
import { SetOverlayReward } from "$lib/rewards/set-overlay";

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
                    new TwitchMessageTrigger({
                        message_contains: "!FurryMode"
                    }),
                    new TwitchBitDonationTrigger({
                        minimum_amount: 500,
                        message_contains: "!FurryMode"
                    }),
                ]
            }),
            rewards: [
                new SetAvatarReward({
                    avatar_id: "avtr_da58f525-347c-4be7-8a26-9dc0ebc83782",
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
                    new TwitchMessageTrigger({
                        message_contains: "!MaidMode"
                    }),
                ]
            }),
            rewards: [
                new SetAvatarReward({
                    avatar_id: "avtr_de75efc5-c67c-4ae8-8a14-cafa07d0fcad",
                    return_to: "previous",
                    timeout_ms: 300000
                })
            ]
        },
        {
            id: crypto.randomUUID(),
            name: "Hide Logo",
            trigger: new TwitchMessageTrigger({
                message_contains: "!HideLogo"
            }),
            rewards: [
                // Hide the logo for 30 seconds
                new SetOverlayReward({
                    overlay_id: 1,
                    show: false,
                    timeout_ms: 30000
                })
            ]
        }
    ],
    baseAvatarId: "avtr_de75efc5-c67c-4ae8-8a14-cafa07d0fcad",
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