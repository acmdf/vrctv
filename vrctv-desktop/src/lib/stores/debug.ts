import type { TriggerSource } from "$lib/triggers/types";
import { writable, type Writable } from "svelte/store";
import type { Service, ServiceStatus } from "../../bindings";

export enum TaskState {
    InProgress = "InProgress",
    Completed = "Completed",
    Failed = "Failed"
}

export const eventLogStore: Writable<TriggerSource[]> = writable([]);
export const serviceStateStore: Writable<Record<Service, ServiceStatus>> = writable({
    "Osc": "Stopped",
    "Overlay": "Stopped",
});
export const taskStateStore: Writable<{ [key: string]: { state: TaskState; reason: string; error?: string; } }> = writable({});