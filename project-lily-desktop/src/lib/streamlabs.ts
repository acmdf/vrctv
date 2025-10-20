import { get } from "svelte/store";
import type { TwitchEventSource } from "../../../project-lily-common/bindings/TwitchEventSource";
import { rewardStore } from "./stores";
import { addReward } from "./rewardHandler";
import type { StreamLabsEvent } from "../../../project-lily-common/bindings/StreamLabsEvent";

interface DonationMessage {
    _id: string;
    amount: number;
    currency: string;
    formattedAmount: string;
    from: string;
    from_user_id: number;
    isPreview: boolean;
    isTest: boolean;
    message: string;
    name: string;
    priority: number;
    recurring: boolean;
    to: {
        name: string;
    };
    unsavedSettings: unknown[];
}

export type StreamLabsEventMatcher = { "type": "donation"; amount?: number; message?: string };

export async function handleStreamlabsEvent(event: StreamLabsEvent) {
    let rewards = get(rewardStore).rewards.filter((r) => r.on.type === "streamlabs");

    for (let reward of rewards) {
        let matches = reward.on.matches as StreamLabsEventMatcher[];

        match: for (let m of matches) {
            if (m.type !== event.type) {
                continue;
            }

            switch (m.type) {
                case "donation":
                    const filter = m as Extract<StreamLabsEventMatcher, { type: "donation" }>;
                    for (const message of (m.message as unknown as DonationMessage[]) ?? []) {
                        if ((message.amount >= (filter.amount ?? 0)) && (message.message.includes(filter.message ?? ""))) {
                            console.log(`Matched donation message ${JSON.stringify(message)} for event ${JSON.stringify(event)}`);
                            addReward(reward);
                        }
                    }
                    // Checked the donation messages, and they might have matched already so for this type we can stop here
                    continue match;
            }

            console.log(`Matched reward ${reward.title} for event ${JSON.stringify(event)}`);
            addReward(reward);
        }
    }
}
