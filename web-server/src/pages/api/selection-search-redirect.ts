import type { APIRoute } from "astro";
export const prerender = false;

function setLastPathSegment(u: string | URL, newSegment: string): URL {
    const url = new URL(u);
    const parts = url.pathname.split("/").filter(Boolean); // remove 0,"", undefined, ...
    parts[parts.length - 1] = newSegment;
    url.pathname = "/" + parts.join("/");
    return url;
}

export const GET: APIRoute = async ({ site, request, url }) => {
    if (site == null) {
        return new Response(null, {
            status: 500,
            statusText: "Missing SITE in .env",
        });
    }

    const resource = url.searchParams.get("resource") || "/";
    const referer = request.headers.get("referer");

    if (referer == null) {
        return new Response(null, {
            status: 400,
            statusText: "Missing referer header",
        });
    }

    let isSameSite = false;
    try {
        const refererURL = new URL(referer);
        const refererOrigin = refererURL.origin;
        isSameSite = refererOrigin === site.origin;

        if (!isSameSite) {
            return new Response(null, {
                status: 400,
                statusText: `Referer url ${refererOrigin} is not on the same site ${site.origin}`,
            });
        }

        const redirectionLocation = setLastPathSegment(refererURL, resource);

        return new Response(null, {
            status: 302,
            headers: {
                Location: redirectionLocation.toString(),
            },
        });
    } catch {
        return new Response(null, {
            status: 400,
            statusText: "Invalid referer url",
        });
    }
};
