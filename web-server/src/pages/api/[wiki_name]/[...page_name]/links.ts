import type { APIRoute } from "astro";
import { get_links, get_number_of_links } from "../../../../db/db";
import { REDIS_PREFIX, redisClient } from "../../../../db/redis";

export const prerender = false;

export type PageLinks = Awaited<ReturnType<typeof get_links>>;

export const PAGINATION_SIZE = 50;

export const GET: APIRoute = async ({ params, url }) => {
    const page_name = params.page_name;
    const wiki_name = params.wiki_name;
    const num = (url.searchParams.get("num") ?? "false") != "false";
    const page = Number(url.searchParams.get("page") ?? "1");

    if (page_name === undefined || wiki_name === undefined) {
        return new Response(null, {
            status: 400,
        });
    }

    let res;

    let identifier = num ? "num_links" : "links";
    const key = `${REDIS_PREFIX}${identifier}:${wiki_name}:${page_name}`;

    const cached_res = await redisClient.get(key);
    // console.log("cached: ", cached_res);

    if (cached_res) {
        res = JSON.parse(cached_res);
        if (num) {
            res = Number(res);
        }
        return new Response(JSON.stringify(res));
    }

    if (num) {
        res = await get_number_of_links(page_name, wiki_name);
    } else {
        res = await get_links(
            page_name,
            wiki_name,
            PAGINATION_SIZE,
            (page - 1) * PAGINATION_SIZE,
        );
    }

    await redisClient.set(key, JSON.stringify(res));

    if (res === undefined) {
        return new Response(null, {
            status: 404,
        });
    }

    return new Response(JSON.stringify(res));
};
