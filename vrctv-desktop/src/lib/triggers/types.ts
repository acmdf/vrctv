import type { RewardContext } from "$lib/rewards/types";
import type { StreamLabsEvent } from "../../../../vrctv-common/bindings/StreamLabsEvent";
import type { TwitchEventSource } from "../../../../vrctv-common/bindings/TwitchEventSource";

export type TriggerSource = TwitchEventSource | StreamLabsEvent;
export type KV = Record<string, string>;

export type Trigger<P> = {
    id: string;
    title: string;
    description: string;

    new(params: Partial<P>): TriggerInstance<P>;
}

export type StoredTrigger = {
    id: string;
    params: any;
};

export class TriggerInstance<P> {
    trigger!: Trigger<P>;

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

    evaluate(_source: TriggerSource): Promise<boolean> {
        throw new Error("Method not implemented.");
    }
    getContext(_context: RewardContext): Promise<KV> {
        throw new Error("Method not implemented.");
    }

    getStoredTrigger(): StoredTrigger {
        return {
            id: this.trigger.id,
            params: this.params
        };
    }
}