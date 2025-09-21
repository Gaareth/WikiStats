import type { APIRoute } from "astro";
import type { NonUndefined, ValuesType } from "utility-types";
import { neighbours } from "../../../../db/db";
import { REDIS_PREFIX, redisClient } from "../../../../db/redis";

export const prerender = false;

export type PageLinks = Awaited<ReturnType<typeof neighbours>>;
export type PageLinksEntry = ValuesType<NonUndefined<PageLinks>>;

type SizeOption = "num_links" | "num_times_linked" | "popularity";

export const GET: APIRoute = async ({ params, url }) => {
    const page_name = params.page_name;
    const wiki_name = params.wiki_name;
    const limit_param = url.searchParams.get("limit");
    type d = "num_links" | "times_linked";
    const sort_param: d = (url.searchParams.get("sort") as d) || "num_links";

    const limit = limit_param != null ? Number(limit_param) : undefined;

    if (
        page_name === undefined ||
        wiki_name === undefined ||
        Number.isNaN(limit) ||
        !["num_links", "times_linked"].includes(sort_param)
    ) {
        return new Response(null, {
            status: 400,
        });
    }

    const key = `${REDIS_PREFIX}:neighbours:${wiki_name}:${page_name}:${sort_param}:${limit}`;
    const cached_res = await redisClient.get(key);

    let res;
    if (cached_res) {
        res = JSON.parse(cached_res);
    } else {
        res = await neighbours(
            page_name,
            limit,
            sort_param!,
            true,
            true,
            wiki_name,
        );

        await redisClient.set(key, JSON.stringify(res));
    }

    if (res === undefined) {
        return new Response(null, {
            status: 404,
        });
    }

    return new Response(JSON.stringify(res));
};
