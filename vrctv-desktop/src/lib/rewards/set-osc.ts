import { get } from "svelte/store";
import { commands, type Result } from "../../bindings";
import { CancellableReward, type RewardContext } from "./types";
import { oscStateStore } from "$lib/stores/global";
import { info } from "@tauri-apps/plugin-log";
import { rewardStore, updateContext } from "$lib/stores/rewards";
import type { KV } from "$lib/triggers/types";

export type SetOSCRewardParams = {
    id: string;
    for_avatar: string;
    params: KV;
    channel_id: string;
    return_to: "previous" | "specific";
    return_params?: KV;
    timeout_ms: number;
}
export class SetOSCReward extends CancellableReward<SetOSCRewardParams> {
    static id = "set-osc-reward";
    static title = "Set OSC Reward";
    static description = "Set OSC parameters for a duration"
    reward = SetOSCReward;

    finishTimeout: number | null = null;
    caughtPreviousParams: KV | null = null;

    constructor(params: Partial<SetOSCRewardParams>) {
        super({
            id: params.id ?? crypto.randomUUID(),
            for_avatar: params.for_avatar || get(rewardStore).baseAvatarId || "",
            params: params.params || {},
            return_to: params.return_to || "previous",
            channel_id: params.channel_id || "",
            return_params: params.return_params || {},
            timeout_ms: params.timeout_ms || 0,
        });
    }

    async validate(): Promise<string | null> {
        return null;
    }

    findPreviousParam(rewards: SetOSCReward[], key: string): string | null {
        for (const reward of rewards) {
            if (reward.params.return_to === "previous" && reward.params.return_params?.[key]) {
                return reward.params.return_params[key];
            }
        }
        return null;
    }

    currentAvatarId(): string {
        const currentAvatarId = get(oscStateStore)["/avatar/change"];

        if (currentAvatarId && "String" in currentAvatarId) {
            return currentAvatarId.String;
        } else {
            return "";
        }
    }

    async readyToStart(context: RewardContext): Promise<boolean> {
        if (this.currentAvatarId() !== this.params.for_avatar) {
            return false;
        }

        let runningRewards = context.runningRewards.filter((r) => r instanceof SetOSCReward);

        if (this.caughtPreviousParams === null && this.params.return_to === "previous") {
            // It will return to the previous params, which works if there isn't a queued SetOSCReward, but if there is, it will try to return to that one instead, so we catch the previous params from this one
            let previousParams: KV = {};

            for (const key of Object.keys(this.params.params)) {
                const previousParam = this.findPreviousParam(runningRewards as SetOSCReward[], key);
                if (previousParam !== null) {
                    previousParams[key] = previousParam;
                }
            }

            this.caughtPreviousParams = previousParams;
        }

        if (this.params.channel_id !== "" && runningRewards.find((r) => r.params.channel_id === this.params.channel_id)) {
            return false;
        }

        return true;
    }

    setParams(params: KV): Promise<Result<null, string>[]> {
        const promises: Promise<Result<null, string>>[] = [];

        for (const [key, value] of Object.entries(params)) {
            promises.push(commands.setOsc(key, value));
        }

        return Promise.all(promises);
    }

    async onStart(context: RewardContext): Promise<void> {
        if (this.params.return_to === "previous") {
            for (const key in this.params.params) {
                if (this.caughtPreviousParams && this.caughtPreviousParams[key]) {
                    this.params.return_params = this.params.return_params || {};
                    this.params.return_params[key] = this.caughtPreviousParams[key];
                } else {
                    const currentValue = get(oscStateStore)[key];

                    if (currentValue) {
                        if ("String" in currentValue) {
                            this.params.return_params = this.params.return_params || {};
                            this.params.return_params[key] = currentValue["String"];
                        } else if ("Bool" in currentValue) {
                            this.params.return_params = this.params.return_params || {};
                            this.params.return_params[key] = currentValue["Bool"] ? "true" : "false";
                        } else if ("Int" in currentValue) {
                            this.params.return_params = this.params.return_params || {};
                            this.params.return_params[key] = currentValue["Int"].toString();
                        } else if ("Float" in currentValue) {
                            this.params.return_params = this.params.return_params || {};
                            this.params.return_params[key] = currentValue["Float"].toString();
                        } else {
                            info(`SetOSCReward: No current value found in OSC state store for key ${key}.`);
                        }
                    }
                }
            }
        }

        await this.setParams(this.params.params);
        if (this.params.timeout_ms > 0) {
            this.finishTimeout = setTimeout(() => this.onCancel(updateContext(context)), this.params.timeout_ms);
        }
    }

    async isStillRunning(_context: RewardContext): Promise<boolean> {
        return this.finishTimeout !== null;
    }

    async onCancel(_context: RewardContext): Promise<void> {
        if (this.finishTimeout !== null) {
            clearTimeout(this.finishTimeout);
            this.finishTimeout = null;
        }

        if (this.params.return_params) {
            await this.setParams(this.params.return_params);
        }

        this.finishCallback?.();
    }
}