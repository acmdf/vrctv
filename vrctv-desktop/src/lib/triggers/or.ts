import type { RewardContext } from "$lib/rewards/types";
import { type KV, TriggerInstance, type TriggerSource } from "./types";

export type OrTriggerParams = {
    subtriggers: TriggerInstance<any>[];
};
export class OrTrigger extends TriggerInstance<OrTriggerParams> {
    static id = "or-trigger";
    static title = "OR Trigger";
    static description = "Group that requires any subtrigger to fire";
    trigger = OrTrigger;

    constructor(params: Partial<OrTriggerParams>) {
        super({
            subtriggers: params.subtriggers ?? [],
        });
    }

    async evaluate(source: TriggerSource): Promise<boolean> {
        for (const trigger of this.params.subtriggers) {
            if (await trigger.evaluate(source)) {
                return true;
            }
        }
        return false;
    }
    async getContext(context: RewardContext): Promise<KV> {
        let combinedKV: KV = {};

        for (const trigger of this.params.subtriggers) {
            if (await trigger.evaluate(context.source)) {
                const kv = await trigger.getContext(context);
                combinedKV = { ...combinedKV, ...kv };
            }
        }

        return combinedKV;
    }
}
