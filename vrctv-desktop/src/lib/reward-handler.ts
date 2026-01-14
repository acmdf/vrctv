import { get } from "svelte/store";
import { CancellableReward, type RewardContext, type RewardInstance } from "./rewards/types";
import type { KV, TriggerSource } from "./triggers/types";
import { rewardStore } from "./stores/rewards";
import { info, warn } from "@tauri-apps/plugin-log";

type TaskContext = {
    kv: KV;
    source: TriggerSource;
}

export class RewardHandler {
    public activeRewards: [RewardInstance<any>, TaskContext][] = [];
    public rewardQueue: [RewardInstance<any>, TaskContext][] = [];
    public globalKV: KV = {};

    constructor() { }

    async handleEvent(event: TriggerSource) {
        let tasks = get(rewardStore).tasks;

        for (const task of tasks) {
            if (await task.trigger.evaluate(event)) {
                info(`Trigger matched for event. Enqueuing rewards.`);
                const context: RewardContext = {
                    source: event,
                    runningRewards: this.activeRewards.map((ar) => ar[0]),
                    rewardQueue: this.rewardQueue.map((rq) => rq[0]),
                    trigger_values: {},
                    global_values: this.globalKV,
                }
                const localKV = await task.trigger.getContext(context);

                for (const reward of task.rewards) {
                    this.rewardQueue.push([reward, { kv: localKV, source: event }]);
                }

                await this.processQueue();
            }
        }
    }

    async cleanseActiveRewards() {
        info(`Cleansing active rewards. Currently active: ${this.activeRewards.length}`);

        const context = {
            runningRewards: this.activeRewards.map((ar) => ar[0]),
            rewardQueue: this.rewardQueue.map((rq) => rq[0]),
            global_values: this.globalKV,
        };

        this.activeRewards = (await Promise.all(this.activeRewards.map(async ([reward, taskContext]) => {
            const stillRunning = await reward.isStillRunning({
                ...context,
                trigger_values: taskContext.kv,
                source: taskContext.source,
            });

            if (!stillRunning) {
                info(`Reward has completed: ${reward.reward.id}, removing from active rewards.`);
            }

            return stillRunning ? [reward, taskContext] as [RewardInstance<any>, TaskContext] : null;
        }))).filter((ar): ar is [RewardInstance<any>, TaskContext] => ar !== null);
    }

    async processQueue() {
        await this.cleanseActiveRewards();

        // Small delay to allow any state changes to propagate
        await new Promise((resolve) => setTimeout(resolve, 50));

        info(`Processing reward queue. Currently queued: ${this.rewardQueue.length}`);

        const context = {
            runningRewards: this.activeRewards.map((ar) => ar[0]),
            rewardQueue: this.rewardQueue.map((rq) => rq[0]),
            global_values: this.globalKV,
        };

        const currentRewardCount = this.activeRewards.length + this.rewardQueue.length;

        for (let i = 0; i < this.rewardQueue.length;) {
            const [reward, taskContext] = this.rewardQueue[i];

            let localContext: RewardContext = {
                ...context,
                trigger_values: taskContext.kv,
                source: taskContext.source,
            };

            if (await reward.readyToStart(localContext)) {
                info(`Starting reward: ${reward.reward.id}`);
                await reward.onStart(localContext);

                if (await reward.isStillRunning(localContext)) {
                    info(`Reward is now active: ${reward.reward.id}`);

                    if (reward instanceof CancellableReward) {
                        reward.finishCallback = async () => {
                            info(`Cancellable reward finished: ${reward.reward.id}`);

                            // Sanity finished check
                            if (await reward.isStillRunning(localContext)) {
                                warn(`Cancellable reward finished callback called but reward is still running: ${reward.reward.id}, with context ${JSON.stringify(localContext)}`);
                            }

                            await this.processQueue();
                        }
                    }
                    this.activeRewards.push([reward, taskContext]);
                }

                this.rewardQueue.splice(i, 1);
            } else {
                i++;
            }
        }

        if (this.activeRewards.length + this.rewardQueue.length < currentRewardCount) {
            // Something changed, re-process the queue
            await this.processQueue();
        }
    }
}