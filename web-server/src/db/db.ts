import Database from "better-sqlite3";
import "dotenv/config";
import { count, eq, like } from "drizzle-orm";
import { drizzle } from "drizzle-orm/better-sqlite3";
import type { PathEntry } from "../components/SP/SPList.tsx";
import { get_supported_wikis } from "./constants.ts";
import * as schema from "./schema.ts";
import { wikiLink, wikiPage } from "./schema.ts";

let db_base_path = process.env.DB_WIKIS_DIR;
console.log("DB PATH [DB_WIKIS_DIR env var]: ", db_base_path);

export const sqlite_connections = Object.fromEntries(
    (await get_supported_wikis("latest")).map((wiki) => [
        wiki,
        new Database(`${db_base_path}/${wiki}_database.sqlite`),
    ]),
);

export const db_connections = Object.fromEntries(
    Object.entries(sqlite_connections).map(([wiki, sqlite]) => [
        wiki,
        drizzle(sqlite, {
            schema,
        }),
    ]),
);

// export const sqlite = new Database(db_path);

// export const db = drizzle(sqlite, {
//   schema,
// });

// export const sqlite_sp_connections = Object.fromEntries(
//     SUPPORTED_WIKIS.map((wiki) => [
//         wiki,
//         new Database(`${db_base_path}/${wiki}_sp_database.sqlite`),
//     ]),
// );

// export const db_sp_connections = Object.fromEntries(
//     Object.entries(sqlite_sp_connections).map(([wiki, sqlite]) => [
//         wiki,
//         drizzle(sqlite, {
//             schema,
//         }),
//     ]),
// );

export async function page_id_to_name(id: number, wiki_name: string) {
    let res = await db_connections[wiki_name].query.wikiPage.findFirst({
        where: eq(wikiPage.pageId, id),
    });
    return res?.pageTitle;
}

export async function page_is_redirect(
    page_title: string,
    wiki_name: string,
): Promise<boolean | undefined> {
    let res = await db_connections[wiki_name].query.wikiPage.findFirst({
        where: eq(wikiPage.pageTitle, page_title),
    });
    return res?.isRedirect == 1;
}

export async function name_to_page_id(name: string, wiki_name: string) {
    // const t1 = new Date().getTime();

    const db = db_connections[wiki_name];
    if (db === undefined) {
        return undefined;
    }

    let res = await db.query.wikiPage.findFirst({
        where: eq(wikiPage.pageTitle, name),
    });

    // console.log(name, wiki_name, "Time: ", new Date().getTime() - t1, "ms");

    return res?.pageId;
}

// export async function get_previous(
//     current_id: number,
//     start_id: number,
//     wiki_name: string,
// ) {
//     let res = await db_sp_connections[wiki_name]
//         .select({ previous_id: spLink.previousId })
//         .from(spLink)
//         .where(
//             and(eq(spLink.pageId, current_id), eq(spLink.sourceId, start_id)),
//         );

//     return res;
// }

export async function get_number_of_links(
    page_name: string,
    wiki_name: string,
) {
    const page_id = await name_to_page_id(page_name, wiki_name);

    if (page_id === undefined) {
        return undefined;
    }

    return (
        await db_connections[wiki_name]
            .select({ value: count() })
            .from(wikiLink)
            .where(eq(wikiLink.pageId, page_id))
    )[0].value;
}

export async function get_links(
    page_name: string,
    wiki_name: string,
    limit: number = 0,
    offset: number = 0,
) {
    // let all_page_ids = db
    //   .select({ pageId: wikiLink.pageLink })
    //   .from(wikiLink)
    //   .leftJoin(wikiPage, eq(wikiLink.pageId, wikiPage.pageId))
    //   .where(
    //     and(
    //       eq(wikiPage.pageTitle, page_name),
    //       where_wiki_name(wikiLink, wiki_name)
    //     )
    //   )
    //   .as("all_page_ids");

    // let res = await db.select({wikiPage}).from(all_page_ids)
    // .leftJoin(wikiPage, eq(all_page_ids.pageId, wikiPage.pageId));
    // console.log(res);

    const page_id = await name_to_page_id(page_name, wiki_name);

    if (page_id === undefined) {
        return undefined;
    }

    // console.log("pageid");
    // console.log(page_name);

    // console.log(page_id);

    // const res = (
    //   await db
    //   // .select()
    //     .select({pageId: wikiPage.pageId, pageTitle: wikiPage.pageTitle, wikiName: wikiPage.wikiName})
    //     .from(wikiLink)
    //     .where(and(
    //       eq(wikiLink.pageId, page_id),
    //       where_wiki_name(wikiLink, wiki_name)
    //     ))
    //     .innerJoin(wikiPage, eq(wikiPage.pageId, wikiLink.pageLink))
    // );

    const ids = await db_connections[wiki_name]
        .select()
        .from(wikiLink)
        .where(eq(wikiLink.pageId, page_id))
        .limit(limit)
        .offset(offset);

    let res = [];
    for (const id of ids) {
        res.push({
            ...id,
            pageTitle: await page_id_to_name(id.pageLink!, wiki_name!),
        });
    }

    return res;
}

export async function get_incoming_links(
    page_name: string,
    wiki_name: string,
    limit: number = 0,
    offset: number = 0,
) {
    const page_id = await name_to_page_id(page_name, wiki_name);

    if (page_id === undefined) {
        return undefined;
    }

    const ids = await db_connections[wiki_name]
        .select()
        .from(wikiLink)
        .where(eq(wikiLink.pageLink, page_id))
        .limit(limit)
        .offset(offset);

    let res = [];
    for (const id of ids) {
        res.push({
            ...id,
            pageTitle: await page_id_to_name(id.pageId!, wiki_name!),
        });
    }

    return res;
}

export async function get_number_of_times_linked(
    page_name: string,
    wiki_name: string,
) {
    const page_id = await name_to_page_id(page_name, wiki_name);

    if (page_id === undefined) {
        return undefined;
    }

    const res = await db_connections[wiki_name]
        .select({ value: count() })
        .from(wikiLink)
        .where(eq(wikiLink.pageLink, page_id));
    return res[0].value;
}

// export async function num_supported_pages(wiki_name: string) {
//     // const res = await db_sp_connections[wiki_name]
//     //   .select({ count: countDistinct(spLink.sourceId) })
//     //   .from(spLink);
//     // return res[0].count;

//     //@ts-ignore
//     const titles = Object.keys(sp_stats[wiki_name]);
//     return titles.length;
// }

export async function neighbours(
    page_name: string,
    limit: number | undefined,
    sort_option: "num_links" | "times_linked",
    num_links: boolean,
    times_linked: boolean,
    wiki_name: string,
) {
    const page_id = await name_to_page_id(page_name, wiki_name);

    if (page_id === undefined) {
        return undefined;
    }

    let binds;
    let limit_str;

    if (limit !== undefined && !Number.isNaN(limit)) {
        binds = { page_id, limit };
        limit_str = "LIMIT @limit";
    } else {
        binds = { page_id };
        limit_str = "";
    }

    const links_sql_str =
        "(select count(*) from WikiLink as wl2 where wl2.page_id = wl1.page_link) as num_links ";
    const linked_sql_str =
        "(select count(*) from WikiLink as wl2 where wl2.page_link = wl1.page_link) as times_linked ";

    //@ts-ignore
    const rows: {
        page_link: number;
        num_links: number | undefined;
        times_linked: number | undefined;
    }[] = sqlite_connections[wiki_name]
        .prepare(
            `select wl1.page_link, ${num_links ? links_sql_str : ""}, ${
                times_linked ? linked_sql_str : ""
            }` +
                `from WikiLink as wl1 where page_id == @page_id order by ${sort_option} desc ${limit_str};`,
        )
        .all(binds);

    let res = [];
    for (const r of rows) {
        res.push({
            ...r,
            pageTitle: (await page_id_to_name(r.page_link!, wiki_name!))!,
        });
    }

    return res;
}

// export async function get_supported_pages(prefix: string, wiki_name: string) {
//     // const all_pages = (
//     //   await db.selectDistinct().from(pageSchema.wikiPage)
//     // ).map((row) => row.pageTitle);

//     // const supported_page_ids = await db_sp_connections[wiki_name]
//     //   .selectDistinct({
//     //     pageId: spLink.sourceId,
//     //   })
//     //   .from(spLink);
//     // // .leftJoin(wikiPage, eq(spLink.sourceId, wikiPage.pageId))
//     // // .where(like(wikiPage.pageTitle, `${prefix}%`))
//     // // .limit(10);

//     // const supported_page_titles: { pageTitle: string }[] = [];
//     // for (const { pageId } of supported_page_ids) {
//     //   supported_page_titles.push({
//     //     pageTitle: (await page_id_to_name(pageId, wiki_name))!,
//     //   });
//     // }

//     //@ts-ignore
//     const titles = Object.keys(sp_stats[wiki_name]);
//     return titles.map((title) => {
//         return { pageTitle: title };
//     });
// }

export async function get_pages_starts_with(prefix: string, wiki_name: string) {
    console.log("Loading get pages");
    console.log(wiki_name);

    let res = await db_connections[wiki_name]
        .select()
        .from(wikiPage)
        .where(like(wikiPage.pageTitle, `${prefix}%`))
        .limit(100);
    console.log("loaded");

    return res;
}

export async function path_entry_from_title(
    page_title: string,
    wiki_name: string,
): Promise<PathEntry> {
    return {
        name: page_title,
        num_links: (await get_number_of_links(page_title, wiki_name))!,
        times_linked: (await get_number_of_times_linked(
            page_title,
            wiki_name,
        ))!,
    };
}

export async function path_from_titles(
    titles: string[],
    wiki_name: string,
): Promise<PathEntry[]> {
    let path = [];
    for (const title of titles) {
        path.push(await path_entry_from_title(title, wiki_name));
    }
    return path;
}

export async function load_link_map(wiki_name: string) {
    console.log("loading link map");

    let links = await db_connections[wiki_name].select().from(wikiLink);
    console.log("loadin? link map");

    let map: Record<number, number[]> = {};
    for (const link of links) {
        if (map[link.pageId!] === undefined) {
            map[link.pageId!] = [];
        }
        map[link.pageId!].push(link.pageLink!);
    }
    console.log("Loaded link map");
    return map;
}

export async function breadth_first_search(
    start_id: number,
    end_id: number,
    wiki_name: string,
) {
    const visit_queue = [start_id];
    const visited = [start_id];

    const prev_map: Record<number, number> = {};
    // const link_map = await load_link_map(wiki_name);

    outer: while (visit_queue.length > 0) {
        const current_page_id: number = visit_queue.shift()!;
        // console.log(current_page_id);

        // let links = link_map[current_page_id];
        // if (links === undefined) {
        //   links = (
        //     await db_connections[wiki_name]
        //       .select({ link_id: wikiLink.pageLink })
        //       .from(wikiLink)
        //       .where(eq(wikiLink.pageId, current_page_id))
        //   ).map((l) => l.link_id!);
        // }
        const links = (
            await db_connections[wiki_name]
                .select({ link_id: wikiLink.pageLink })
                .from(wikiLink)
                .where(eq(wikiLink.pageId, current_page_id))
        ).map((l) => l.link_id!);

        for (const link of links) {
            // const link_id = link.link_id!;
            const link_id = link;

            if (!visited.includes(link_id)) {
                // console.log(link_id);

                // console.log(link_id);
                prev_map[link_id] = current_page_id;
                visited.push(link_id);

                if (link_id == end_id) {
                    console.log("found: " + end_id);
                    break outer;
                }

                visit_queue.push(link_id);
            }
        }
    }

    const build_path = async () => {
        const end_title = await page_id_to_name(end_id!, wiki_name);
        const path = [end_title!];

        let current_id: number = end_id!;
        while (true) {
            const previous = prev_map[current_id];
            // console.log(previous);

            if (previous === undefined) {
                break;
            }

            current_id = previous;
            const title = await page_id_to_name(current_id, wiki_name);
            path.push(title!);
        }
        return path;
    };

    return build_path();
}

// breadth_first_search("Taylor_Swift", "Zupfinstrument", "dewiki");

// export async function get_precalced_shortest_path(
//     start_id: number,
//     end_title: string,
//     end_id: number,
//     wiki_name: string,
// ) {
//     const path = [];

//     let current: string = end_title;
//     let current_id: number = end_id;

//     while (current != undefined) {
//         let path_entry: PathEntry = {
//             name: current,
//             num_links: (await get_number_of_links(current, wiki_name))!,
//             times_linked: (await get_number_of_times_linked(
//                 current,
//                 wiki_name,
//             ))!,
//         };

//         path.push(path_entry);
//         console.log(current);

//         let prev = await get_previous(current_id, start_id, wiki_name);

//         // when there is no previous, either you at the start or no path calculated or unexpected error
//         if (prev.length <= 0) {
//             break;
//         }

//         current_id = prev[0].previous_id;
//         current = (await page_id_to_name(current_id, wiki_name))!;
//     }
//     path.reverse();
//     return path;
// }
