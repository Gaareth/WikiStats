import type { APIRoute } from "astro";
import { page_is_redirect } from "../../../../db/db";

export const prerender = false;

export const GET: APIRoute = async ({ params }) => {
    const page_name = params.page_name;
    const wiki_name = params.wiki_name;

    if (page_name === undefined || wiki_name === undefined) {
        return new Response(null, {
            status: 400,
        });
    }

    const res = await page_is_redirect(page_name, wiki_name);

    if (res === undefined) {
        return new Response(null, {
            status: 404,
        });
    }

    return new Response(JSON.stringify(res));
};
