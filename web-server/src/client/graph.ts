import Graph from "graphology";
import graphml from "graphology-graphml/browser";
import type { PageLinks } from "../pages/api/[wiki_name]/[...page_name]/neighbours";
import { clamp } from "../utils";

export async function fromGraphmlEndpoint(path: string) {
    const resp = await fetch(path);

    if (!resp.ok) {
        if (resp.status == 404) {
            let msg = (await resp.json()).statusText;
            throw Error(msg);
        }

        throw Error(resp.statusText);
    }

    const fileContents = await resp.text();
    return graphml.parse(Graph, fileContents);
}

export async function fromPageFetch(wiki_name: string, page_title: string) {
    const graph = new Graph();
    graph.addNode(page_title, {
        label: page_title,
        a: "a",
        size: 10,
        color: "gray",
        x: Math.random() * 50,
        y: Math.random() * 20,
    });

    const getNeighbours = async (name: string) => {
        return await getNeighboursFetch(wiki_name, name, undefined);
    };

    await addNeighboursDFS(page_title, graph, getNeighbours, 1);
    return graph;
}

export function resizeNodes(graph: Graph, sizeFn: (node: string) => number) {
    graph.forEachNode((node, _) => {
        // console.log(sizeFn(node));

        graph.setNodeAttribute(node, "size", sizeFn(node));
    });
}

export async function resizeNodesAsync(
    graph: Graph,
    sizeFn: (node: string) => Promise<number>,
) {
    for (const node of graph.nodeEntries()) {
        graph.setNodeAttribute(node.node, "size", await sizeFn(node.node));
    }
}

// https://www.learnui.design/tools/data-color-picker.html
export const LEVEL_COLORS = [
    "#003f5c",
    "#374c80",
    "#7a5195",
    "#bc5090",
    "#ef5675",
    "#ff764a",
    "#ffa600",
];

export const addNeighboursDFS = async (
    parent: string,
    graph: Graph,
    getNeighbours: (parent: string) => Promise<PageLinks>,
    current_depth: number | undefined = 1,
    settings: {
        max_depth?: number;
        calcNodeSize?: (arg1: any) => Promise<number>;
    } = {},
) => {
    const {
        max_depth = 1,
        calcNodeSize = (arg1: any) => clamp(arg1.num_links, 2, 20),
    } = settings;

    const neighbours = (await getNeighbours(parent))!;

    for (const link of neighbours) {
        const { pageTitle, num_links, times_linked } = link;

        if (!graph.nodes().includes(pageTitle)) {
            graph.addNode(pageTitle, {
                label: pageTitle,
                size: await calcNodeSize(link),
                color: LEVEL_COLORS[current_depth],
                x: Math.random() * 50,
                y: Math.random() * 20,
                num_links,
                times_linked,
            });

            if (current_depth < max_depth) {
                addNeighboursDFS(
                    pageTitle,
                    graph,
                    getNeighbours,
                    current_depth + 1,
                    settings,
                );
            }
        }

        if (!graph.neighbors(parent).includes(pageTitle)) {
            graph.addEdge(parent, pageTitle, {
                size: 1,
                color: LEVEL_COLORS[current_depth - 1],
            });
        }
    }
};

export const getNeighboursFetch = async (
    wiki_name: string,
    title: string,
    limit: number | undefined,
) => {
    const limit_str = limit !== undefined ? `?limit=${limit}` : "";
    const resp = await fetch(
        `/api/${wiki_name}/${title}/neighbours${limit_str}`,
    );
    const json: PageLinks = await resp.json();

    return json;
};
