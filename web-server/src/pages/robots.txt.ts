import type { APIRoute } from "astro";

const getRobotsTxt = (sitemapURL: URL) => `\
User-agent: *
Disallow: /wiki/*/mirror/

Sitemap: ${sitemapURL.href}
`;

export const GET: APIRoute = ({ site }) => {
    try {
        const sitemapURL = new URL("sitemap-index.xml", site);
        return new Response(getRobotsTxt(sitemapURL));
    } catch (e) {
        console.log("Failed building url for robots.txt", e, "Site:", site);
        throw e;
    }
};
