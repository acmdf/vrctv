import type { RewardContext } from "$lib/rewards/types";
import { TriggerInstance, type KV, type TriggerSource } from "./types";
import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";


type targetEvent = Extract<TwitchEventSource, { type: "BitDonation" }>;

export type TwitchBitDonationTriggerParams = {
    minimum_amount?: number;
    message_contains?: string;
}
export class TwitchBitDonationTrigger extends TriggerInstance<TwitchBitDonationTriggerParams> {
    static id = "twitch-bit-donation-trigger";
    static title = "Twitch Bit Donation Trigger";
    static description = "Twitch Bit Donation Trigger";
    trigger = TwitchBitDonationTrigger;

    async evaluate(source: TriggerSource): Promise<boolean> {
        if (source.type === "BitDonation") {
            const event = source as targetEvent;

            if (this.params.minimum_amount && this.params.minimum_amount > event.amount) {
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
        if (context.source.type !== "BitDonation") {
            return {};
        }

        return {
            donation_amount: (context.source as targetEvent).amount.toString(),
            donation_message: (context.source as targetEvent).message || "",
        }
    }
}