import type { TriggerSource } from "$lib/triggers/types";
import type { Component } from "svelte";

export type RewardContext = {
    source: TriggerSource;
    runningRewards: RewardInstance<any>[];
    rewardQueue: RewardInstance<any>[];
    trigger_values: {
        [key: string]: string;
    };
    global_values: {
        [key: string]: string;
    };
};

export type StoredReward = {
    id: string;
    params: any;
};

export type Reward<P> = {
    id: string;
    title: string;
    description: string;

    new(params: Partial<P>): RewardInstance<P>;
}

export class RewardInstance<P> {
    reward!: Reward<P>;

    private _params!: P;
    public get params(): P {
        return this._params;
    }
    public set params(value: P) {
        this._params = value;
    }

    constructor(params: P) {
        this.params = params;
    }

    // Function called to check if the reward is ready to start, e.g. if another reward is still active that would conflict
    // Run when the reward is first added to the queue, and after every any other reward finishes until it returns true
    readyToStart(_context: RewardContext): Promise<boolean> {
        throw new Error("Method not implemented.");
    }

    // Function called when the reward is started
    onStart(_context: RewardContext): Promise<void> {
        throw new Error("Method not implemented.");
    }

    // Function called to check if a reward is still running, e.g. for time-limited rewards
    // Run after adding, and after any other reward is started or finished
    isStillRunning(_context: RewardContext): Promise<boolean> {
        throw new Error("Method not implemented.");
    }

    // Check that the reward's content is valid, e.g. avatar ID exists etc., and return an error message if not
    validate(): Promise<string | null> {
        throw new Error("Method not implemented.");
    }

    getStoredReward(): StoredReward {
        return {
            id: this.reward.id,
            params: this.params
        };
    }
}

export class CancellableReward<P> extends RewardInstance<P> {
    onCancel(_context: RewardContext): Promise<void> {
        throw new Error("Method not implemented.");
    }

    finishCallback?: () => Promise<void>;
}
