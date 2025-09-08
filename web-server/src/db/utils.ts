import { eq } from "drizzle-orm";

export async function make_wiki_stat_record<V>(
    fn: (name: string) => Promise<V>,
    wiki_names: { name: string }[],
) {
    let record: Record<string, V> = {};
    // for (const { name } of wiki_names) {
    //   console.log("calling fn for wiki; " + name);

    //   record[name] = await fn(name);
    // }
    wiki_names.forEach(async ({ name }) => {
        console.log("calling fn for wiki; " + name);
        record[name] = await fn(name);
    });
    return record;
}

export function get_unwrap_or<K extends string | number | symbol, V>(
    access: K | undefined,
    data: Record<K, V>,
    data_or: V,
) {
    if (access === undefined) {
        return data_or;
    } else {
        return data[access];
    }
}

export function where_wiki_name(
    sqltable: { wikiName: any },
    wiki_name: string | undefined = undefined,
) {
    return wiki_name ? eq(sqltable.wikiName, wiki_name) : undefined;
}
