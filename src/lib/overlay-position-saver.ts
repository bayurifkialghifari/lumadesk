const POSITION_SAVE_DELAY_MS = 180;

type SavePosition<T> = (position: T) => Promise<void>;
type ReportPositionError = (error: unknown) => void;

export class LatestPositionSaver<T> {
  readonly #save: SavePosition<T>;
  readonly #reportError: ReportPositionError;
  readonly #delayMs: number;
  #pending: T | undefined;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #saving = false;
  #disposed = false;

  public constructor(
    save: SavePosition<T>,
    reportError: ReportPositionError,
    delayMs = POSITION_SAVE_DELAY_MS,
  ) {
    this.#save = save;
    this.#reportError = reportError;
    this.#delayMs = delayMs;
  }

  public schedule(position: T): void {
    if (this.#disposed) return;
    this.#pending = position;
    if (this.#timer !== undefined) clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#timer = undefined;
      void this.#flush();
    }, this.#delayMs);
  }

  public dispose(): void {
    this.#disposed = true;
    this.#pending = undefined;
    if (this.#timer !== undefined) clearTimeout(this.#timer);
    this.#timer = undefined;
  }

  async #flush(): Promise<void> {
    if (this.#saving || this.#disposed) return;
    this.#saving = true;
    try {
      while (!this.#disposed && this.#pending !== undefined) {
        const position = this.#pending;
        this.#pending = undefined;
        try {
          await this.#save(position);
        } catch (error) {
          this.#reportError(error);
        }
      }
    } finally {
      this.#saving = false;
    }
  }
}
