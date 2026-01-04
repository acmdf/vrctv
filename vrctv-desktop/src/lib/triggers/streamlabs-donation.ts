import type { RewardContext } from "$lib/rewards/types";
import type { KnownStreamlabsEvents } from "$lib/streamlabs";
import { TriggerInstance, type KV, type Trigger, type TriggerSource } from "./types";

type targetEvent = Extract<KnownStreamlabsEvents, { type: "donation" }>;

export type StreamlabsDonationTriggerParams = {
    minimum_amount?: number;
    message_contains?: string;
}
export class StreamlabsDonationTrigger extends TriggerInstance<StreamlabsDonationTriggerParams> {
    static id = "streamlabs-donation-trigger";
    static title = "Streamlabs Donation Trigger";
    static description = "Trigger for Streamlabs donations";
    trigger = StreamlabsDonationTrigger;

    matchedMessage(message: targetEvent["message"]): targetEvent["message"][0] | null {
        for (const msg of message) {
            if (this.params.minimum_amount && msg.amount < this.params.minimum_amount) {
                continue;
            }
            if (this.params.message_contains && !msg.message.includes(this.params.message_contains)) {
                continue;
            }
            return msg;
        }
        return null;
    }

    async evaluate(source: TriggerSource): Promise<boolean> {
        if (source.type !== "donation") {
            return false;
        }

        // The streamlabs api is quite undocumented, so this might break at some point
        const event = source as unknown as targetEvent;

        const matched = this.matchedMessage(event.message);

        return matched !== null;
    }

    async getContext(context: RewardContext): Promise<KV> {
        if (context.source.type !== "donation") {
            return {};
        }

        const event = context.source as unknown as targetEvent;
        const matched = this.matchedMessage(event.message);

        if (!matched) {
            return {};
        }

        return {
            donation_amount: matched.amount.toString(),
            donation_message: matched.message,
            donation_currency: matched.currency,
            donation_from: matched.from,
        };
    }
}