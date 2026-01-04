import { derived, get, writable } from "svelte/store";
import { commands, type Avatar } from "../bindings";
import toast from "svelte-french-toast";

const avatarListCache = writable<Avatar[]>([]);
const avatarLastFetchTime = writable<number>(0);
const avatarOscCache = writable<Record<string, string[]>>({});
const avatarOscLastFetchTime = writable<Record<string, number>>({});
const CACHE_DURATION_MS = 5 * 60 * 1000; // 5 minutes

export const cachedAvatarStore = derived<[typeof avatarListCache, typeof avatarLastFetchTime], Avatar[]>(
    [avatarListCache, avatarLastFetchTime],
    ([$avatarListCache, $avatarLastFetchTime], set) => {
        const now = Date.now();

        if ($avatarListCache && (now - $avatarLastFetchTime) < CACHE_DURATION_MS) {
            return; // Value is fresh, no need to fetch
        }

        avatarLastFetchTime.set(Date.now());
        commands.fetchAvatars().then((avatars) => {
            if (avatars.status === "error") {
                toast.error("Failed to fetch avatars: " + avatars.error);
                return;
            }

            avatarListCache.set(avatars.data);
            set(avatars.data);
        });

        return; // Initial return to avoid undefined
    },
    []
)

export async function getAvatarOscs(avatarId: string): Promise<string[]> {
    const now = Date.now();

    let cache = get(avatarOscCache);
    let lastFetchTimes = get(avatarOscLastFetchTime);

    if (cache[avatarId] && lastFetchTimes[avatarId] && (now - lastFetchTimes[avatarId]) < CACHE_DURATION_MS) {
        return cache[avatarId]; // Return cached value
    }

    lastFetchTimes[avatarId] = now;
    avatarOscLastFetchTime.set(lastFetchTimes);
    const response = await commands.fetchAvatarOsc(avatarId);

    if (response.status === "error") {
        toast.error("Failed to fetch avatar OSCs: " + response.error);
        return [];
    }
    cache[avatarId] = response.data;

    avatarOscCache.set(cache);

    return response.data;
}