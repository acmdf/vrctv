import type { RewardContext } from "$lib/rewards/types";
import { TriggerInstance, type KV, type TriggerSource } from "./types";
import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";

type targetEvent = Extract<TwitchEventSource, { type: "Message" }>;

export type TwitchMessageTriggerParams = {
    sender?: string;
    message_contains?: string;
}
export class TwitchMessageTrigger extends TriggerInstance<TwitchMessageTriggerParams> {
    static id = "twitch-message-trigger";
    static title = "Twitch Message Trigger";
    static description = "Twitch Message Trigger";
    trigger = TwitchMessageTrigger;

    async evaluate(source: TriggerSource): Promise<boolean> {
        if (source.type === "Message") {
            const event = source as targetEvent;

            if (this.params.sender && this.params.sender !== event.sender) {
                return false;
            }

            if (this.params.message_contains && !event.message?.includes(this.params.message_contains)) {
                return false;
            }

            return true;
        }
        return false;
    }

    async getContext(context: RewardContext): Promise<KV> {
        if (context.source.type !== "Message") {
            return {};
        }

        return {
            message_sender: (context.source as targetEvent).sender,
            message: (context.source as targetEvent).message,
        }
    }
}