import { WIKIPEDIA_REST_API_HEADERS } from "./utils";

export type SiteInfo = {
    url: string;
    dbname: string;
    code: string;

    sitename: string;
    name?: string;
    localname?: string;
}

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
        }
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