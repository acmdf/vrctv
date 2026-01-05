import { commands, type Result } from "../../bindings";
import { CancellableReward, type RewardContext } from "./types";
import type { KV } from "$lib/triggers/types";
import { debug, info } from "@tauri-apps/plugin-log";
import { updateContext } from "$lib/stores/rewards";

export type SetWarudoOscRewardParams = {
    params: KV;
    channel_id: string;
    timeout_ms: number;
    return_params: KV;
}
export class SetWarudoOscReward extends CancellableReward<SetWarudoOscRewardParams> {
    static id = "set-warudo-osc-reward";
    static title = "Set Warudo OSC Reward";
    static description = "Set warudo osc parameters for a duration"
    reward = SetWarudoOscReward;

    finishTimeout: number | null = null;

    constructor(params: Partial<SetWarudoOscRewardParams>) {
        super({
            params: params.params ?? {},
            channel_id: params.channel_id ?? "",
            return_params: params.return_params ?? {},
            timeout_ms: params.timeout_ms ?? 0,
        });
    }

    async validate(): Promise<string | null> {
        return null;
    }

    async readyToStart(context: RewardContext): Promise<boolean> {
        let runningRewards = context.runningRewards.filter((r) => r instanceof SetWarudoOscReward);

        if (this.params.channel_id !== "" && runningRewards.find((r) => r.params.channel_id === this.params.channel_id)) {
            return false;
        }

        return true;
    }

    setParams(params: KV): Promise<Result<null, string>[]> {
        info(`Setting Warudo OSC params: ${JSON.stringify(params)}`);

        const promises: Promise<Result<null, string>>[] = [];

        for (const [key, value] of Object.entries(params)) {
            promises.push(commands.setWarudoOsc(key, value));
        }

        return Promise.all(promises);
    }

    async onStart(context: RewardContext): Promise<void> {
        await this.setParams(this.params.params);

        if (this.params.timeout_ms > 0) {
            this.finishTimeout = setTimeout(() => this.onCancel(updateContext(context)), this.params.timeout_ms);
        }
    }

    async isStillRunning(_context: RewardContext): Promise<boolean> {
        return this.finishTimeout !== null;
    }

    async onCancel(context: RewardContext): Promise<void> {
        if (this.finishTimeout !== null) {
            clearTimeout(this.finishTimeout);
            this.finishTimeout = null;
        }

        // Check for queued rewards that will overwrite return params
        let fictionalContext: RewardContext = {
            runningRewards: context.runningRewards.filter((r) => r !== this),
            rewardQueue: context.rewardQueue,
            global_values: context.global_values,
            trigger_values: {},
            source: context.source
        };

        const queuedRewards = context.rewardQueue.filter((r) => r instanceof SetWarudoOscReward).filter((r) => r.readyToStart(fictionalContext));
        let nonCollidingReturnParams: KV = {};

        for (const [key, value] of Object.entries(this.params.return_params)) {
            if (!queuedRewards.find((r) => Object.keys(r.params.params).includes(key))) {
                nonCollidingReturnParams[key] = value;
            } else {
                debug(`Not returning param ${key} for reward ${this.reward.id} as it will be overwritten by a queued reward.`);
            }
        }

        await this.setParams(nonCollidingReturnParams);

        this.finishCallback?.();
    }
}