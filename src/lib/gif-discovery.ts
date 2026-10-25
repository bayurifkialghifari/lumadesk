const TENOR_HOME_URL = "https://tenor.com/";
const TENOR_SEARCH_URL = "https://tenor.com/search/";

export function gifDiscoveryUrl(search: string): URL {
  const words = search
    .trim()
    .split(/\s+/u)
    .filter((word) => word.length > 0);
  if (words.length === 0) return new URL(TENOR_HOME_URL);

  const slug = encodeURIComponent(words.join("-"));
  return new URL(`${TENOR_SEARCH_URL}${slug}-gifs`);
}
