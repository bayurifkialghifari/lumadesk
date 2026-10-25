export const OVERLAY_CONTROLS_IDLE_MS = 3_000;

export type OverlayControlsHold = "focus" | "pointer";

export class OverlayControlsVisibility {
  private readonly holds = new Set<OverlayControlsHold>();
  private hideTimer: ReturnType<typeof setTimeout> | undefined;
  private visible = false;

  public constructor(
    private readonly onChange: (visible: boolean) => void,
    private readonly idleMs = OVERLAY_CONTROLS_IDLE_MS,
  ) {}

  public reveal(): void {
    if (!this.visible) {
      this.visible = true;
      this.onChange(true);
    }
    this.restartIdleTimer();
  }

  public activity(): void {
    if (this.visible) this.restartIdleTimer();
  }

  public hold(reason: OverlayControlsHold): void {
    this.holds.add(reason);
    this.reveal();
  }

  public release(reason: OverlayControlsHold): void {
    this.holds.delete(reason);
    this.restartIdleTimer();
  }

  public dispose(): void {
    this.clearIdleTimer();
    this.holds.clear();
  }

  private restartIdleTimer(): void {
    this.clearIdleTimer();
    if (!this.visible || this.holds.size > 0) return;
    this.hideTimer = setTimeout(() => {
      this.hideTimer = undefined;
      this.visible = false;
      this.onChange(false);
    }, this.idleMs);
  }

  private clearIdleTimer(): void {
    if (this.hideTimer !== undefined) clearTimeout(this.hideTimer);
    this.hideTimer = undefined;
  }
}
