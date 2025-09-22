import type { APIRoute } from "astro";
import { getTasks } from "../../db/redis-utils";
export const prerender = false;

export const GET: APIRoute = async ({ params, url }) => {
    const tasks = await getTasks();
    const is_updating = tasks.some((t) => t.status === "RUNNING");

    return new Response(JSON.stringify({ is_updating }));
};
