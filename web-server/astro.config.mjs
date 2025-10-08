import node from "@astrojs/node";
import solidJs from "@astrojs/solid-js";
import tailwind from "@astrojs/tailwind";
import icon from "astro-icon";
import { defineConfig } from "astro/config";

import sitemap from "@astrojs/sitemap";
import { loadEnv } from "vite";

const env = loadEnv(process.env.NODE_ENV, process.cwd(), '');

console.log("process", process.env.SITE);
console.log("meta", import.meta.env.SITE);

console.log("env", env.SITE);


// https://astro.build/config
export default defineConfig({
    integrations: [solidJs(), tailwind(), icon(), sitemap()],
    output: "static",
    adapter: node({
        mode: "standalone",
    }),
    site: process.env.SITE,
});
