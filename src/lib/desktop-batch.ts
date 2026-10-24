export interface BatchOutcome {
  readonly total: number;
  readonly succeeded: number;
  readonly failed: number;
}

export async function runSettledBatch<T>(
  items: readonly T[],
  operation: (item: T) => Promise<unknown>,
): Promise<BatchOutcome> {
  const results = await Promise.allSettled(
    items.map((item) => Promise.resolve().then(() => operation(item))),
  );
  const failed = results.filter((result) => result.status === "rejected").length;
  return { total: items.length, succeeded: items.length - failed, failed };
}
