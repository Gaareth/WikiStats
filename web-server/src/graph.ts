import fs from "fs";
import Graph from "graphology";
import graphml from "graphology-graphml";

export function fromGraphmlFile(path: string) {
    const fileContents = fs.readFileSync(path).toString();
    return graphml.parse(Graph, fileContents);
}

export async function fromPage(wiki_name: string, page_title: string) {
    const graph = new Graph();
    graph.addNode(page_title, {
        label: page_title,
        a: "a",
        size: 10,
        color: "gray",
        x: Math.random() * 50,
        y: Math.random() * 20,
    });

    // const getNeighbours = (wiki_name: string, name: string) => biggest_neighbours(name, 10, wiki_name);
    // await addNeighbours(wiki_name, page_title, graph, getNeighbours);
    return graph;
}
