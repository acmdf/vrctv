import { get } from "svelte/store";
import type { RewardContext, RewardInstance } from "./rewards/types";
import type { KV, TriggerSource } from "./triggers/types";
import { rewardStore } from "./stores/rewards";

type TaskContext = {
    kv: KV;
    source: TriggerSource;
}

export class RewardHandler {
    private activeRewards: [RewardInstance<any>, TaskContext][] = [];
    private rewardQueue: [RewardInstance<any>, TaskContext][] = [];
    private globalKV: KV = {};

    constructor() { }

    async handleEvent(event: TriggerSource) {
        let tasks = get(rewardStore).tasks;

        for (const task of tasks) {
            if (await task.trigger.evaluate(event)) {
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
        const context = {
            runningRewards: this.activeRewards.map((ar) => ar[0]),
            rewardQueue: this.rewardQueue.map((rq) => rq[0]),
            global_values: this.globalKV,
        };

        this.activeRewards = await Promise.all(this.activeRewards.filter(async ([reward, taskContext]) => {
            const stillRunning = await reward.isStillRunning({
                ...context,
                trigger_values: taskContext.kv,
                source: taskContext.source,
            });
            return stillRunning;
        }));
    }

    async processQueue() {
        await this.cleanseActiveRewards();

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
                await reward.onStart(localContext);

                if (await reward.isStillRunning(localContext)) {
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