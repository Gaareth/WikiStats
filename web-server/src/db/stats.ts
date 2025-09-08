// export async function num_pages_loaded(
//   wiki_name: string | undefined = undefined
// ): Promise<number> {
//   let res = await db
//     .select({
//       value: sql<number>`count('*')`,
//     })
//     .from(wikiPage)
//     .where(where_wiki_name(wikiPage, wiki_name));
//   return res[0].value;
// }

// export async function max_min_num_pages_loaded() {
//   let res = await db
//     .select({ wikiName: wikiPage.wikiName, count: count() })
//     .from(wikiPage)
//     .groupBy(wikiPage.wikiName)
//     .orderBy(desc(count()));

//   return { max: res[0], min: res.slice(-1)[0] };
// }

// export async function max_min_num_links_loaded() {
//   let res = await db
//     .select({ wikiName: wikiLink.wikiName, count: count() })
//     .from(wikiLink)
//     .groupBy(wikiLink.wikiName)
//     .orderBy(desc(count()));

//   return { max: res[0], min: res.slice(-1)[0] };
// }

// // console.log(await max_num_pages_loaded());

// export async function num_links_loaded(
//   wiki_name: string | undefined
// ): Promise<number> {
//   console.log("Loading num links");

//   let res = await db
//     .select({
//       value: count(),
//     })
//     .from(wikiLink)
//     .where(where_wiki_name(wikiLink, wiki_name));
//   console.log("LOADED num links");

//   return res[0].value;
// }

// export async function get_dead_pages(wiki_name: string | undefined): Promise<{
//   id: number | null;
//   count: number;
//   wiki_name: string;
//   pageTitle: string;
// }[]> {
//   let res = await db
//     .select({
//       id: wikiLink.pageId,
//       count: sql<number>`count()`,
//       wiki_name: wikiLink.wikiName,
//     })
//     .from(wikiLink)
//     .where(where_wiki_name(wikiLink, wiki_name))
//     .groupBy(wikiLink.pageId)
//     .having(({ count }) => eq(count, 0));

//   let titles = await add_title(res);
//   //@ts-ignore
//   return titles;
// }

// export async function get_root_pages(wiki_name: string | undefined): Promise<{
//   id: number | null;
//   count: number;
//   wiki_name: string;
//   pageTitle: string;
// }[]> {
//   let res = await db
//     .select({
//       id: wikiLink.pageId,
//       count: sql<number>`count()`,
//       wiki_name: wikiLink.wikiName,
//     })
//     .from(wikiLink)
//     .where(where_wiki_name(wikiLink, wiki_name))
//     .groupBy(wikiLink.pageLink)
//     .having(({ count }) => eq(count, 0));

//   let titles = await add_title(res);
//   //@ts-ignore
//   return titles;
// }

// export async function most_links(
//   limit: number,
//   wiki_name: string | undefined = undefined,
//   second_ordering: any = null
// ): Promise<LinkCount[]> {
//   // let res = await db
//   //   .select({
//   //     id: wikiLink.pageId,
//   //     title: wikiPage.pageTitle,
//   //     count: sql<number>`count()`,
//   //     wiki_name: wikiPage.wikiName,
//   //   })
//   //   .from(wikiLink)
//   //   .where(where_wiki_name(wikiLink, wiki_name))
//   //   .leftJoin(wikiPage, eq(wikiLink.pageId, wikiPage.pageId))
//   //   .groupBy(wikiLink.pageId)
//   //   .orderBy(desc(sql`count()`), second_ordering)
//   //   .limit(limit);

//   let res = await db
//     .select({
//       id: wikiLink.pageId,
//       count: sql<number>`count()`,
//       wiki_name: wikiLink.wikiName,
//     })
//     .from(wikiLink)
//     .groupBy(wikiLink.pageId)
//     .orderBy(desc(sql`count()`), second_ordering)
//     .limit(limit);

//   const result: LinkCount[] = [];
//   for (const r of res) {
//     const title = (await page_id_to_name(r.id!))!;
//     result.push({ ...r, title });
//   }
//   return result;
// }

// export async function add_title(input: any[]) {
//   const result: any[] = [];
//   for (const r of input) {
//     const title = (await page_id_to_name(r.id!))!;
//     result.push({ ...r, pageTitle: title });
//   }
//   return result;
// }

// export async function most_linked(
//   limit: number,
//   wiki_name: string | undefined = undefined
// ): Promise<LinkCount[]> {
//   // sqlite.prepare("SELECT page_title, page_link, COUNT(*) FROM WikiLink JOIN WikiPage ON WikiPage.page_id = WikiLink.page_link where WikiLink.wiki_name = 'ja' GROUP BY page_link ORDER BY count(*) DESC LIMIT 10;");

//   let res = await db
//     .select({
//       id: wikiLink.pageLink,
//       count: sql<number>`count()`,
//       wiki_name: wikiLink.wikiName,
//     })
//     .from(wikiLink)
//     .groupBy(wikiLink.pageLink)
//     .orderBy(desc(sql`count()`))
//     .limit(limit);

//   const result: LinkCount[] = [];
//   for (const r of res) {
//     const title = (await page_id_to_name(r.id!))!;
//     result.push({ ...r, title });
//   }

//   return result;
// }

export type LinkCount = {
    id: number | null;
    title: string | null;
    count: number;
    wiki_name: string | null;
};

// export const select_page_order_by = async (
//   wiki_name: string | undefined = undefined,
//   ordering: SQL<unknown>
// ) => {
//   let res = await db
//     .select()
//     .from(wikiPage)
//     .where(where_wiki_name(wikiPage, wiki_name))
//     .orderBy(ordering)
//     .limit(1);
//   return res[0];
// };

// const select_page_grouped_by_order_by = async (
//   grouping: SQL<unknown>,
//   ordering: SQL<unknown>
// ) => {
//   let res = await db
//     .select()
//     .from(wikiPage)
//     .groupBy(grouping)
//     .orderBy(ordering)
//     .limit(1);
//   return res[0].pageTitle;
// };
