import { commands, type Result } from "../../bindings";
import { CancellableReward, type RewardContext } from "./types";
import type { KV } from "$lib/triggers/types";

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
        const promises: Promise<Result<null, string>>[] = [];

        for (const [key, value] of Object.entries(params)) {
            promises.push(commands.setWarudoOsc(key, value));
        }

        return Promise.all(promises);
    }

    async onStart(context: RewardContext): Promise<void> {
        await this.setParams(this.params.params);

        if (this.params.timeout_ms > 0) {
            this.finishTimeout = setTimeout(() => this.onCancel(context), this.params.timeout_ms);
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

        await this.setParams(this.params.return_params);

        this.finishCallback?.();
    }
}