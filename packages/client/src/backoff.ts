/** Delay before reconnect attempt `attempt` (0-based): exponential, capped, with jitter. */
export function backoffDelay(
  attempt: number,
  {
    base = 500,
    max = 15_000,
    random = Math.random,
  }: { base?: number; max?: number; random?: () => number } = {},
): number {
  const exp = Math.min(max, base * 2 ** attempt);
  // Full jitter between half and all of the exponential delay.
  return Math.round(exp / 2 + (random() * exp) / 2);
}
