/** What `GET /api/a/hello` answers. */
export interface Hello {
  message: string;
  /** Greetings application-a has given so far, this one included. */
  count: number;
}

/**
 * Greet `name` through application-a, at `base` — the page's own origin
 * when empty.
 */
export async function hello(name: string, base = ""): Promise<Hello> {
  const response = await fetch(
    `${base}/api/a/hello?name=${encodeURIComponent(name)}`,
  );
  if (!response.ok) {
    throw new Error(`application-a answered ${response.status}`);
  }
  return (await response.json()) as Hello;
}
