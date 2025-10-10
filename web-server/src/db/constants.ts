import { getCollection, getEntry, type InferEntrySchema } from "astro:content";
import { parseDumpDate, type KeysOfType } from "../utils";

export const WIKI_NAMES = (await get_supported_wikis("latest")).map((n) => {
    return { name: n };
});
// export const ALL_PAGES = await get_pages();
// export const NUM_SUPPORTED_SP_PAGES = await make_wiki_stat_record(num_supported_pages, WIKI_NAMES);
// export const NUM_SUPPORTED_SP_PAGES = (];

type WikiStat<T> = {
    data_per_wiki: Record<string, T>;
    data_all: T;
    get: (wiki_name: string | undefined) => T;
};

type Stats = InferEntrySchema<"stats">;
type RecordKeys = KeysOfType<Stats, Record<string, any>>;
type ValueOf<T> = T[keyof T];

export async function get_sorted_entries(order: "asc" | "desc") {
    const collection = await getCollection("stats");
    if (collection.length == 0) {
        return [];
    }

    collection.sort(
        (c1, c2) =>
            (parseDumpDate(c1.id).getTime() - parseDumpDate(c2.id).getTime()) *
            (order == "desc" ? -1 : 1),
    );

    return collection;
}

export async function get_latest_date(): Promise<string | undefined> {
    const collection = await get_sorted_entries("desc");
    return collection[0].id;
}

export async function get_supported_wikis(dump_date: string) {
    const dump_date_validated =
        dump_date == "latest" ? await get_latest_date() : dump_date;
    if (dump_date_validated == null) {
        return [];
    }
    const entry = await getEntry("stats", dump_date_validated);
    return entry?.data.wikis ?? [];
}

export type Trend = {
    trend: "up" | "down" | "no change";
    absValue: number;
    relValue: number;
};

// export function get_trend_from_all(all: any[]): Trend<any> | undefined {

// }

// todo: better typeing
export function get_trend_link(wiki_name: string | undefined, stat: string) {
    return `/wiki/${wiki_name != null ? wiki_name + "/" : ""}stat/${stat}`;
}

export function get_trend_from_all_number(
    all: number[],
): [current: number, trend: Trend | undefined] {
    const current = all.at(-1)!;
    const prev = all.at(-2);
    const trend = get_trend(current, prev);

    return [current, trend];
}

export function get_trend(
    current_data: number,
    previous_data: number | undefined,
): Trend | undefined {
    if (previous_data == null) {
        return undefined;
    }

    const absValue = current_data - previous_data;
    const relValue = previous_data !== 0 ? (absValue / previous_data) * 100 : 0;

    return {
        trend: absValue > 0 ? "up" : absValue < 0 ? "down" : "no change",
        absValue,
        relValue,
    };
}
// , ValueOf<Stats[key]> | undefined]
export async function make_wiki_stat<key extends RecordKeys>(key: key) {
    const extract = (
        entry_data: InferEntrySchema<"stats"> | undefined,
        wiki_name: string | undefined,
    ) => {
        if (entry_data == null) {
            return undefined;
        }
        const data_current = entry_data[key];

        if (data_current == null) {
            return undefined;
        }

        const wiki_name_or_global = wiki_name ?? "global";
        if (
            typeof data_current === "object" &&
            data_current !== null &&
            (data_current as Record<string, any>).hasOwnProperty(
                wiki_name_or_global,
            )
        ) {
            return (data_current as Record<string, any>)[wiki_name_or_global];
        }

        // for global stats that dont have per wiki data
        if (wiki_name == undefined) {
            return data_current;
        }

        // if wikiname is not in the data return undefined
        return undefined;
    };

    async function get(
        dump_date: "latest" | string,
        wiki_name?: string | undefined,
    ): Promise<ValueOf<Stats[key]>> {
        const data_current = await get_stat(dump_date);
        return extract(data_current, wiki_name);
    }

    async function get_all_until(
        dump_date: "latest" | string,
        wiki_name?: string | undefined,
        add_undefined?: boolean,
    ): Promise<ValueOf<Stats[key]>[]> {
        // const data_current = await get_stat(dump_date);
        const stats = await get_stats_until(dump_date);
        if (stats == null) {
            return [];
        }

        const extracted_stats: ValueOf<Stats[key]>[] = [];
        for (const stat of stats) {
            const extracted_data = extract(stat, wiki_name);
            // only add non undefined data, to still calculate trends even if the directly previous dumpdate does not exit
            // or add undefined if wanted
            if (extracted_data != null || add_undefined) {
                extracted_stats.push(extracted_data);
            }
        }
        return extracted_stats;
    }

    return {
        get,
        get_all_until,
    };
}

// export function make_wiki_stat<T extends Record<string, any>>(
//     data: T,
// ): { get: (name: string | undefined) => (typeof data)["global"] } {
//     let global_data = data["global"];
//     const get = (wiki_name: string | undefined) =>
//         get_unwrap_or(wiki_name, data, global_data);

//     return {
//         get,
//     };
// }

export const NUM_PAGES_LOADED_STAT = await make_wiki_stat("num_pages");
export const NUM_REDIRECTS_LOADED_STAT = await make_wiki_stat("num_redirects");
export const NUM_LINKED_REDIRECTS_STAT = await make_wiki_stat(
    "num_linked_redirects",
);

export const NUM_LINKS_LOADED_STAT = await make_wiki_stat("num_links");

export const TOP_TEN_MOST_LINKED_STAT = await make_wiki_stat("most_linked");
// let b = TOP_TEN_MOST_LINKED_STAT.get("global");
// TOP_TEN_MOST_LINKED_STAT.get("")
// export const TOP_TEN_MOST_LINKED_STAT = "most_linked"
export const TOP_TEN_MOST_LINKS_STAT = await make_wiki_stat("most_links");

export const LONGEST_NAME_STAT = await make_wiki_stat("longest_name");
export const LONGEST_NO_REDIRECTS_NAME_STAT = await make_wiki_stat(
    "longest_name_no_redirect",
);

export const NUM_DEAD_PAGES_STAT = await make_wiki_stat("num_dead_pages");

export const NUM_ROOT_PAGES_STAT = await make_wiki_stat("num_orphan_pages");
export const NUM_DEAD_ROOT_PAGES_STAT = await make_wiki_stat(
    "num_dead_orphan_pages",
);

export async function get_stat(dump_date: string) {
    const dump_date_validated =
        dump_date == "latest" ? await get_latest_date() : dump_date;

    if (dump_date_validated == null) {
        return undefined;
    }

    const entry = await getEntry("stats", dump_date_validated);
    const data = entry?.data;
    return data;
}

export async function get_stats_until(dump_date: string) {
    const dump_date_validated =
        dump_date == "latest" ? await get_latest_date() : dump_date;

    if (dump_date_validated == null) {
        return undefined;
    }

    const entries = await get_sorted_entries("asc");

    const stats = [];
    const dump_date_time = parseDumpDate(dump_date_validated).getTime();

    for (const entry of entries) {
        if (parseDumpDate(entry.id).getTime() <= dump_date_time) {
            stats.push(entry.data);
        }
    }

    return stats;
}

export const MAX_NUM_PAGES_STAT = await make_wiki_stat("max_num_pages");
export const MIN_NUM_PAGES_STAT = await make_wiki_stat("min_num_pages");

export const MAX_NUM_LINKS_STAT = await make_wiki_stat("max_num_links");
export const MIN_NUM_LINKS_STAT = await make_wiki_stat("min_num_links");
