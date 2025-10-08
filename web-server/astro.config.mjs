import node from "@astrojs/node";
import solidJs from "@astrojs/solid-js";
import tailwind from "@astrojs/tailwind";
import icon from "astro-icon";
import { defineConfig } from "astro/config";

import sitemap from "@astrojs/sitemap";


// https://astro.build/config
export default defineConfig({
    integrations: [solidJs(), tailwind(), icon(), sitemap()],
    output: "static",
    adapter: node({
        mode: "standalone",
    }),
    site: process.env.SITE,
});
