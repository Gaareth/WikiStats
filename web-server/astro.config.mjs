import node from "@astrojs/node";
import solidJs from "@astrojs/solid-js";
import tailwind from "@astrojs/tailwind";
import icon from "astro-icon";
import { defineConfig } from "astro/config";

import sitemap from "@astrojs/sitemap";
import { loadEnv } from "vite";

// https://github.com/withastro/astro/issues/12667
const env = loadEnv(process.env.NODE_ENV, process.cwd(), "");

// https://astro.build/config
export default defineConfig({
    integrations: [solidJs(), tailwind(), icon(), sitemap()],
    output: "static",
    adapter: node({
        mode: "standalone",
    }),
    site: env.SITE,
});
