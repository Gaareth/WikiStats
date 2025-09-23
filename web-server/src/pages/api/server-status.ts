import type { APIRoute } from "astro";
import { CELERY_REDIS_PREFIX, redisClient } from "../../db/redis";
import { getTasks } from "../../db/redis-utils";
export const prerender = false;

export const GET: APIRoute = async () => {
    const tasks = await getTasks();
    const is_updating = tasks.some((t) => t.status === "RUNNING");
    const is_rebuilding =
        (await redisClient.get(`${CELERY_REDIS_PREFIX}:is-rebuilding`)) ===
        "true";

    return new Response(JSON.stringify({ is_updating, is_rebuilding }));
};
