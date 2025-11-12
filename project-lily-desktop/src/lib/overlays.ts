import type { OverlayItem } from "../bindings";

export function currentOverlayState(overlaysVisible: Record<number, boolean>, overlays: OverlayItem[]): Array<OverlayItem> {
    return Object.values(overlays).map((overlay) => ({
        ...overlay,
        visible: overlaysVisible[overlay.id] || overlay.visible,
    }));
}
