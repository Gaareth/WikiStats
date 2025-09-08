// import { asc, desc, sql } from "drizzle-orm";
// import { get_wiki_names } from "./db";
// import {
//   most_linked,
//   most_links,
//   select_page_order_by,
//   type LinkCount,
//   get_dead_pages,
//   num_pages_loaded,
//   num_links_loaded,
//   max_min_num_pages_loaded,
//   max_min_num_links_loaded,
//   get_root_pages,
// } from "./stats";
// import { get_unwrap_or, make_wiki_stat_record } from "./utils";

// export const WIKI_NAMES = await get_wiki_names();
// // export const WIKI_NAMES = [{name: "de"}];

// type WikiStat<T> = {
//   data_per_wiki: Record<string, T>;
//   data_all: T;
//   get: (wiki_name: string | undefined) => T;
// };

// export async function make_wiki_stat<V>(
//   fn: (name: string | undefined) => Promise<V>
// ): Promise<WikiStat<V>> {
//   let t1 = performance.now();
//   console.log("making stat");

//   try {
//     console.log(fn.toString().split("function")[1].split(" {")[0]);
//   } catch {
//     console.log(fn.toString());
//   }

//   const [data_all, data_per_wiki] = await Promise.all([fn(undefined), make_wiki_stat_record(fn, WIKI_NAMES)]);
//   // const data_all = await fn(undefined);
//   // let data_per_wiki: Record<string, V> = {"ja": data_all};

//   const get = (wiki_name: string | undefined) =>
//     get_unwrap_or(wiki_name, data_per_wiki, data_all);
//   console.log("made: elapsed: " + (performance.now() - t1)/1000);

//   return {
//     data_per_wiki,
//     data_all,
//     get,
//   };
// }

// export const NUM_PAGES_LOADED_STAT = await make_wiki_stat(num_pages_loaded);
// export const NUM_LINKS_LOADED_STAT = await make_wiki_stat(num_links_loaded);

// export const TOP_TEN_MOST_LINKED_STAT = await make_wiki_stat(
//   async (name) => await most_linked(10, name)
// );

// export const TOP_TEN_MOST_LINKS_STAT = await make_wiki_stat(
//   async (name) => await most_links(10, name)
// );

// export const LONGEST_NAME_STAT = await make_wiki_stat(
//   async (name) =>
//     await select_page_order_by(name, desc(sql`length(page_title)`))
// );

// // export const SHORTEST_NAME_MOST_LINKS_STAT = await make_wiki_stat(
// //   async (name) => await most_links(1, undefined, desc(sql`length(page_title)`))
// // );

// export const SHORTEST_NAME_STAT = await make_wiki_stat(
//   async (name) =>
//     await select_page_order_by(name, asc(sql`length(page_title)`))
// );

// export const DEAD_PAGES_STAT = await make_wiki_stat(
//   async (name) =>
//     await get_dead_pages(name)
// );

// export const ROOT_PAGES_STAT = await make_wiki_stat(
//   async (name) =>
//     await get_root_pages(name)
// );

// const NUM_PAGES_LOADED_GLOBAL =  await max_min_num_pages_loaded();
// export const MAX_NUM_PAGES_STAT = NUM_PAGES_LOADED_GLOBAL.max;
// export const MIN_NUM_PAGES_STAT = NUM_PAGES_LOADED_GLOBAL.min

// const NUM_LINKS_LOADED_GLOBAL =  await max_min_num_links_loaded();
// export const MAX_NUM_LINKS_STAT = NUM_LINKS_LOADED_GLOBAL.max;
// export const MIN_NUM_LINKS_STAT = NUM_LINKS_LOADED_GLOBAL.min;
