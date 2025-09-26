export const WIKI_TYPES = [
    "wiki",
    "wiktionary",
    "wikinews",
    "wikisource",
    "wikiquote",
    "wikivoyage",
    "wikibooks",
    "wikiversity",
    "wikimedia",
];

export const WIKIPEDIA_REST_API_HEADERS = new Headers({
    "Api-User-Agent": import.meta.env.WIKIPEDIA_REST_API_USER_AGENT,
});
