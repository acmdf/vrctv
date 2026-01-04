import { writable, type Writable } from "svelte/store";
import type { OverlayItem } from "../../bindings";
import { persisted } from "svelte-persisted-store";

export const overlays: Writable<OverlayItem[]> = persisted(
    "overlaysStore",
    []
);
export const overlayVisibleStore: Writable<Record<number, boolean>> = writable({});