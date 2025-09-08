import type { APIRoute } from "astro";
import {
    get_incoming_links,
    get_number_of_times_linked,
} from "../../../../db/db";
import { REDIS_PREFIX, redisClient } from "../../../../db/redis";
import { PAGINATION_SIZE } from "./links";

export const prerender = false;

export type PageLinked = Awaited<ReturnType<typeof get_number_of_times_linked>>;

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

    let identifier = num ? "num_linked" : "linked";
    const key = `${REDIS_PREFIX}${identifier}:${wiki_name}:${page_name}`;
    const cached_res = await redisClient.get(key);
    if (cached_res) {
        res = JSON.parse(cached_res);
        if (num) {
            res = Number(res);
        }
        return new Response(JSON.stringify(res));
    }

    if (num) {
        res = await get_number_of_times_linked(page_name, wiki_name);
    } else {
        res = await get_incoming_links(
            page_name,
            wiki_name,
            PAGINATION_SIZE,
            (page - 1) * PAGINATION_SIZE,
        );
    }

    if (res === undefined) {
        return new Response(null, {
            status: 404,
        });
    }
    await redisClient.set(key, JSON.stringify(res));

    return new Response(JSON.stringify(res));
};
