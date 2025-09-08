import type { APIRoute } from "astro";
import { parse_stream } from "../../../components/SP/stream";
import { REDIS_PREFIX, redisClient } from "../../../db/redis";
export const prerender = false;

export const GET: APIRoute = async ({ params, url }) => {
    const wiki_name = params.wiki_name;
    const start = url.searchParams.get("start");
    const end = url.searchParams.get("end");
    const stream = (url.searchParams.get("stream") ?? "false") != "false";

    if (wiki_name === undefined || start == null || end == null) {
        return new Response(null, {
            status: 400,
        });
    }

    const sp_url = `${process.env.SP_SERVER}/path/${wiki_name}?start_title=${start}&end_title=${end}&stream=${stream}`;

    const redis_key = `${REDIS_PREFIX}:shortest_path:${wiki_name}:${start}:${end}`;
    const cached_res = await redisClient.get(redis_key);

    if (cached_res) {
        return new Response(cached_res);
    }

    try {
        const res = await fetch(sp_url);
        if (!res.ok || res.body == null) {
            return new Response(await res.text(), { status: res.status });
        }

        const reader = res.clone().body!.getReader();
        let json_line;

        parse_stream(reader, async (text_line) => {
            json_line = JSON.parse(text_line);
            if (json_line.paths !== undefined) {
                await redisClient.set(redis_key, text_line);
            }
        });

        return res;
    } catch (error: any) {
        console.log(error);
        console.log(error.toString());
        console.log("Cant reach backend server at: " + sp_url);

        let message = error.toString();
        if (error instanceof TypeError) {
            message = "Can't reach backend server";
        }

        return new Response(message, { status: 500 });
    }
};
