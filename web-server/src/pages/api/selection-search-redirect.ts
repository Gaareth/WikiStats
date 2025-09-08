import type { APIRoute } from "astro";
export const prerender = false;

export const GET: APIRoute = async ({ url, ...rest }) => {
    const resource = url.searchParams.get("resource") || "/";

    return new Response(null, {
        status: 302,
        headers: {
            Location: resource,
        },
    });
};
