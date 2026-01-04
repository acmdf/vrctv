import type { RewardContext } from "$lib/rewards/types";
import { TriggerInstance, type KV, type TriggerSource } from "./types";
import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";

type targetEvent = Extract<TwitchEventSource, { type: "Whisper" }>;

export type TwitchWhisperTriggerParams = {
    sender?: string;
    message_contains?: string;
}
export class TwitchWhisperTrigger extends TriggerInstance<TwitchWhisperTriggerParams> {
    static id = "twitch-whisper-trigger";
    static title = "Twitch Whisper Trigger";
    static description = "Twitch Whisper Trigger";
    trigger = TwitchWhisperTrigger;

    async evaluate(source: TriggerSource): Promise<boolean> {
        if (source.type === "Whisper") {
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
        if (context.source.type !== "Whisper") {
            return {};
        }

        return {
            message_sender: (context.source as targetEvent).sender,
            message: (context.source as targetEvent).message,
        }
    }
}