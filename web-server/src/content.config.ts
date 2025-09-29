import { glob } from "astro/loaders";
import { defineCollection, getCollection } from "astro:content";
import statsSchema from "./schemas/stats-schema";

const statsCollection = defineCollection({
    loader: glob({ pattern: "**/*.json", base: "./data/stats" }),
    schema: statsSchema,
});

export const collections = {
    stats: statsCollection,
};

export async function getEarlierEntry(dumpDate: string) {
    const collection = await getCollection("stats");
    if (dumpDate == "latest") {
        return collection.length == 1
            ? undefined
            : collection[collection.length - 2];
    }

    let idx = collection.findIndex((c) => c.id == dumpDate);
    if (idx == -1 || idx == 0) {
        return undefined;
    }

    return collection[idx - 1];
}

// export async function get_earlier_dump_date(dumpDate: string) {
//     return getEarlierEntry(dumpDate).then((e) => e?.id);
// }
