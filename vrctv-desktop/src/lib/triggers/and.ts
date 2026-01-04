import type { RewardContext } from "$lib/rewards/types";
import { type KV, TriggerInstance, type TriggerSource } from "./types";

export type AndTriggerParams = {
    subtriggers: TriggerInstance<any>[];
}
export class AndTrigger extends TriggerInstance<AndTriggerParams> {
    static id = "and-trigger";
    static title = "AND Trigger";
    static description = "Group that requires all subtriggers to fire";
    trigger = AndTrigger;

    constructor(params: Partial<AndTriggerParams>) {
        super({
            subtriggers: params.subtriggers ?? [],
        });
    }

    async evaluate(source: TriggerSource): Promise<boolean> {
        for (const trigger of this.params.subtriggers) {
            if (!await trigger.evaluate(source)) {
                return false;
            }
        }
        return true;
    }
    async getContext(context: RewardContext): Promise<KV> {
        let combinedKV: KV = {};

        for (const trigger of this.params.subtriggers) {
            const kv = await trigger.getContext(context);
            combinedKV = { ...combinedKV, ...kv };
        }

        return combinedKV;
    }
}