import type { APIRoute } from "astro";
import fs from "fs";
import path from "path";
import { get_pages_starts_with } from "../../../db/db";
export const prerender = false;
export type Pages = Awaited<ReturnType<typeof get_pages_starts_with>>;

export const GET: APIRoute = async ({ params }) => {
    const wiki_name = params.wiki_name;
    if (wiki_name == null) {
        return new Response(null, { status: 404 });
    }

    let graph_dir = process.env.GRAPHS_WIKIS_DIR;

    if (graph_dir == null) {
        return new Response(
            JSON.stringify({
                statusText: "process.env.GRAPHS_WIKIS_DIR is undefined",
            }),
            {
                status: 500,
            },
        );
    }

    let graph_path = path.join(
        graph_dir,
        wiki_name + "-most-popular-100.graphml",
    );

    try {
        const fileContents = fs.readFileSync(graph_path).toString();
        return new Response(fileContents);
    } catch {
        if (!fs.existsSync(graph_path)) {
            return new Response(
                JSON.stringify({
                    statusText:
                        "Graphml file for wiki " + wiki_name + " was not found",
                }),
                {
                    status: 404,
                },
            );
        }

        return new Response(null, { status: 500 });
    }
};
