import Graph from "graphology";
import { createSignal, onMount } from "solid-js";
import {
    fetch_number_of_links,
    fetch_number_times_linked,
} from "../../client/api";
import { LEVEL_COLORS } from "../../client/graph";
import { SigmaProvider } from "../Sigma/SigmaContext";
import {
    DEFAULT_RENDER_SETTINGS,
    GraphView,
    SizeOption,
} from "../Sigma/SigmaGraph";

interface SPProps {
    wiki_name: string;
    paths: string[][];
}

export default function SPGraph(props: SPProps) {
    let graph = new Graph({ type: "directed" });
    // let [graph, setGraph] = createSignal(new Graph({ type: "directed" }));
    const [loaded, setLoaded] = createSignal(false);

    let DEPTH_MAP: Record<string, string[]> = {};
    for (let index = 0; index < props.paths[0].length; index++) {
        // DEPTH_COUNT[index] = 0;
        DEPTH_MAP[index] = [];
    }

    const add_path_to_graph = (path: string[]) => {
        let previous = undefined;
        let depth = 0;

        for (const page_title of path) {
            let color = LEVEL_COLORS[depth];
            if (depth == path.length - 1) {
                color = "#ef4444";
            } else if (depth == 0) {
                color = "#22c55e";
            }

            if (!graph.nodes().includes(page_title)) {
                graph.addNode(page_title, {
                    label: page_title,
                    size: 100,
                    color,
                    x: depth * 1,
                    y: 0,
                    // num_links: 10,
                    // times_linked: 10,
                });

                // DEPTH_COUNT[depth] += 1;
                DEPTH_MAP[depth].push(page_title);
            }

            if (
                previous !== undefined &&
                !graph.neighbors(previous).includes(page_title)
            ) {
                graph.addEdge(previous, page_title, {
                    size: 1,
                    color,
                });
            }

            previous = page_title;
            depth += 1;
        }
    };

    const GAP = 0.07;
    const layout_graph = async () => {
        for (const [depth_s, nodes] of Object.entries(DEPTH_MAP)) {
            const depth = Number(depth_s);

            let offset = (nodes.length / 2) * GAP;
            if (nodes.length == 1) {
                offset = 0;
            }

            let i = 0;
            for (const node of nodes) {
                const num_outbound_neighbors =
                    graph.outboundNeighbors(node).length;
                graph.setNodeAttribute(
                    node,
                    "outgoing",
                    num_outbound_neighbors,
                );
                graph.setNodeAttribute(
                    node,
                    "num_links",
                    await fetch_number_of_links(node, props.wiki_name),
                );

                graph.setNodeAttribute(
                    node,
                    "times_linked",
                    await fetch_number_times_linked(node, props.wiki_name),
                );

                // to much force rendered labels result in visual clutter
                // || node.length < 5
                if (nodes.length < 25 || num_outbound_neighbors > 15) {
                    graph.setNodeAttribute(node, "forceLabel", true);
                }

                if (depth == props.paths[0].length - 1 || depth == 0) {
                    graph.setNodeAttribute(node, "y", 0);
                    graph.setNodeAttribute(node, "forceLabel", true);
                } else {
                    const y = i * GAP - offset;
                    graph.setNodeAttribute(node, "y", y);
                }
                i += 1;
            }
        }
    };

    onMount(async () => {
        for (const path of props.paths) {
            add_path_to_graph(path);
        }
        await layout_graph();

        setLoaded(true);
    });

    // const [graphR] = createResource(async () => {
    //   for (const path of props.paths) {
    //     add_path_to_graph(path);
    //   }

    //   // await layout_graph();
    // });

    return (
        <>
            <section class="my-10">
                <h2 class="text-2xl -mb-7">All {props.paths.length} paths</h2>
                <SigmaProvider
                    graph={graph}
                    loaded={loaded}
                    renderSettings={{
                        ...DEFAULT_RENDER_SETTINGS,
                        possible_size_options: {
                            ...DEFAULT_RENDER_SETTINGS.possible_size_options,
                            outgoing: "Outgoing Links",
                        },
                        size_by: SizeOption.outgoing,
                    }}>
                    <GraphView
                        wiki_name={props.wiki_name}
                        showLayoutToggler={false}
                    />
                </SigmaProvider>
            </section>
        </>
    );
}
