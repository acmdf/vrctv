import type { RewardContext } from "$lib/rewards/types";
import { TriggerInstance, type KV, type TriggerSource } from "./types";
import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";

type targetEvent = Extract<TwitchEventSource, { type: "ChannelPoints" }>;

export type TwitchChannelPointsTriggerParams = {
    reward_id?: string;
}
export class TwitchChannelPointsTrigger extends TriggerInstance<TwitchChannelPointsTriggerParams> {
    static id = "twitch-channel-points-trigger";
    static title = "Twitch Channel Points Trigger";
    static description = "Twitch Channel Points Trigger";
    trigger = TwitchChannelPointsTrigger

    async evaluate(source: TriggerSource): Promise<boolean> {
        if (source.type === "ChannelPoints") {
            const event = source as targetEvent;

            if (this.params.reward_id && this.params.reward_id !== event.reward_id) {
                return false;
            }

            return true;
        }
        return false;
    }

    async getContext(context: RewardContext): Promise<KV> {
        if (context.source.type !== "ChannelPoints") {
            return {};
        }

        return {
            reward_id: (context.source as targetEvent).reward_id || "",
            reward_name: (context.source as targetEvent).reward_name || "",
        }
    }
}