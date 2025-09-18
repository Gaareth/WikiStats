import type { APIRoute } from "astro";
import { get_pages_starts_with } from "../../../db/db";
import { REDIS_PREFIX, redisClient } from "../../../db/redis";
export const prerender = false;

export type Pages = Awaited<ReturnType<typeof get_pages_starts_with>>;

export const GET: APIRoute = async ({ params, url }) => {
    // return new Response(
    //   JSON.stringify([{pageTitle: "test"}])
    // );

    const wiki_name = params.wiki_name;
    const prefix = url.searchParams.get("prefix") || "";
    const sp_start = (url.searchParams.get("sp_start") ?? "false") != "false";

    if (wiki_name === undefined) {
        return new Response(null, {
            status: 400,
        });
    }

    let pages;

    const key = `${REDIS_PREFIX}pages:${wiki_name}:${prefix}`;
    const cached_res = await redisClient.get(key);
    if (cached_res) {
        pages = JSON.parse(cached_res);
    } else {
        pages = await get_pages_starts_with(prefix, wiki_name);
        await redisClient.set(key, JSON.stringify(pages));
    }

    // if (sp_start) {
    //   const supported_page_titles = await get_supported_pages(prefix, wiki_name);
    //   pages = supported_page_titles;
    // } else {
    //   pages = await get_pages_starts_with(prefix, wiki_name);
    // }

    if (pages === undefined) {
        return new Response(null, {
            status: 404,
        });
    }

    return new Response(JSON.stringify(pages));
};
