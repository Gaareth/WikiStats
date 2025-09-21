import type { Pages } from "./pages/api/[wiki_name]/pages";
import { WIKIPEDIA_REST_API_HEADERS } from "./utils";

export type SiteInfo = {
    url: string;
    dbname: string;
    code: string;

    sitename: string;
    name?: string;
    localname?: string;
};

export type SiteMatrixResult = {
    sitematrix: {
        count: number;
        [key: number]: {
            code: string;
            name: string;
            site: {
                url: string;
                dbname: string;
                code: string;
                sitename: string;
                closed: boolean;
            }[];
            dir: string;
            localname: string;
        };
        specials: {
            [key: number]: {
                url: string;
                dbname: string;
                code: string;
                lang: string;
                sitename: string;
                closed?: boolean;
                private?: string;
                fishbowl?: string; // only to logged in?
            };
        };
    };
};

export async function fetchSiteMatrix() {
    const base_url = `https://en.wikipedia.org/w/api.php`;

    const resp = await fetch(
        `${base_url}?action=sitematrix&format=json&formatversion=2`,
        {
            headers: WIKIPEDIA_REST_API_HEADERS,
        },
    );
    const json: SiteMatrixResult = await resp.json();
    if (json == undefined) {
        throw new Error("Failed to fetch site matrix. Resp: " + resp.status);
    }

    return json.sitematrix;
}

export type QueryResult = {
    query: {
        redirects: [
            {
                index: number;
                from: string;
                to: string;
                tofragment?: string;
            },
        ];
        pages: [
            {
                pageid: number;
                ns: number;
                title: string;
                index: number;
                terms?: {
                    description: [string];
                };
            },
        ];
    };
};

export type Page = {
    title: string;
    description: string;
};

export const fetchPages = async (wikiName: string, prefix: string) => {
    try {
        const resp = await fetch(`/api/${wikiName}/pages?prefix=${prefix}`);
        const json: Pages = await resp.json();
        return json.map((o) => o.pageTitle);
    } catch {
        return [];
    }
};

export const fetchPagesWikipediaAPI = async (
    wiki_name: string,
    prefix: string,
): Promise<Page[]> => {
    // await sleep(5000);
    const lang_prefix = wiki_name.substring(0, 2);
    const base_url = `https://${lang_prefix}.wikipedia.org/w/api.php`;
    const num_results = 10;

    const resp = await fetch(
        `${base_url}?action=query&format=json&
        gpssearch=${prefix}&
        generator=prefixsearch&
        prop=pageprops|pageterms&
        redirects=1&
        wbptterms=description&
        gpsnamespace=0&
        gpslimit=${num_results}&
        formatversion=2&
        origin=*`,
        {
            headers: WIKIPEDIA_REST_API_HEADERS,
        },
    );
    const json: QueryResult = await resp.json();
    if (json.query == undefined) {
        return [];
    }

    const result = json.query.pages.map(({ title, terms }) => ({
        title,
        description: terms?.description[0] ?? "-",
    }));

    return result;
};

export const fetchRandomPage = async (wiki_name: string) => {
    const lang_prefix = wiki_name.substring(0, 2);

    const resp = await fetch(
        `https://${lang_prefix}.wikipedia.org/api/rest_v1/page/random/title`,
        {
            headers: WIKIPEDIA_REST_API_HEADERS,
        },
    );

    const json = await resp.json();

    if (!resp.ok) {
        // https://phabricator.wikimedia.org/T364153
        const title = resp.url.split(
            `https://${lang_prefix}.wikipedia.org/api/rest_v1/page/title/`,
        )[1];
        return decodeURIComponent(title);
    }

    const title = json["items"][0]["title"];
    const namespace = json["items"][0]["namespace"];
    if (namespace != 0) {
        throw new Error(
            "Not a main namespace page: " +
                title +
                " but this should not happen",
        );
    }
    return title;
};
