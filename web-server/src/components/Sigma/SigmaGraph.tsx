import Graph from "graphology";
import ForceSupervisor from "graphology-layout-force/worker";
import Sigma from "sigma";
import themeStore from "../ThemeStore";

import type { Attributes } from "graphology-types";
import {
    EdgeArrowProgram,
    EdgeLineProgram,
    EdgeRectangleProgram,
} from "sigma/rendering";
import type { EdgeDisplayData, NodeDisplayData } from "sigma/types";
import {
    Show,
    createEffect,
    createSignal,
    onCleanup,
    onMount,
    type Component,
} from "solid-js";
import {
    addNeighboursDFS,
    fromGraphmlEndpoint,
    getNeighboursFetch,
    resizeNodes,
} from "../../client/graph";
import { range, wiki_link } from "../../utils";
import { type ClusterProps } from "./GraphCluster";
import GraphController, { type ControllerProps } from "./GraphController";
import GraphPanels from "./GraphPanels";
import { type SettingsProps } from "./GraphSettings";
import { SigmaProvider, useSigma } from "./SigmaContext";
import { drawHover } from "./canvas-util";

interface Props {
    showClusters?: boolean;
    showLayoutToggler?: boolean;
    autoStartLayout?: boolean;
    wiki_name: string;
    // graph2: Graph;
    graphFetch?: () => Promise<Graph>;
}

export interface State {
    selectedNode?: string;
    searchQuery: string;

    // State derived from query:
    searchedNode?: string;
    suggestions?: Set<string>;

    // State derived from hovered node:
    selectedNeighbors?: Set<string>;

    selectedClusterID?: string;
}

// cluster definition
export interface Cluster {
    // label: string;
    // x?: number;
    // y?: number;
    color?: string;
    // positions: { x: number; y: number }[];
    id: string;
    num_nodes: number;
}

export interface GraphRenderSettings {
    neighbors: number;
    depth: number;
    max_size: number;
    min_size: number;
    size_by: SizeOption;
    // size_scale: number;
    possible_size_options: Record<string, string>;
}

export enum SizeOption {
    times_linked = "times_linked",
    num_links = "num_links",
    outgoing = "outgoing",
}

export const DEFAULT_RENDER_SETTINGS: GraphRenderSettings = {
    depth: 2,
    neighbors: 10,
    max_size: 25,
    min_size: 5,
    size_by: SizeOption.num_links,
    // size_scale: 1 / 2,
    possible_size_options: {
        num_links: "Links",
        times_linked: "Times linked",
    },
};

export function GraphView({
    wiki_name,
    showClusters = false,
    showLayoutToggler = false,
    autoStartLayout = false,
    // graph,
    graphFetch,
}: Props) {
    const { getGraph, setGraph, loaded } = useSigma()!["graph"];
    const { renderSettings, setRenderSettings } = useSigma()!["renderSettings"];

    const { theme } = themeStore;
    // const [getGraph, setGraph] = useSigma();

    let container: any;
    let renderer: Sigma;
    let layout: ForceSupervisor<Attributes, Attributes>;
    let [clusters, setClusters] = createSignal(new Map<string, Cluster>());

    const [graphState, setGraphState] = createSignal<State>({
        searchQuery: "",
    });
    const [layoutIsRunning, setLayoutIsRunning] = createSignal(autoStartLayout);
    const [loadingError, setLoadingError] = createSignal<string | undefined>();

    function setHoveredNode(node?: string) {
        if (getGraph() === undefined || renderer === undefined) {
            return;
        }

        let state = graphState();
        if (node) {
            state.selectedNode = node;
            state.selectedNeighbors = new Set(getGraph().neighbors(node));
        }

        // Compute the partial that we need to re-render to optimize the refresh
        const nodes = getGraph().filterNodes(
            (n) => n !== state.selectedNode && !state.selectedNeighbors?.has(n),
        );
        const nodesIndex = new Set(nodes);
        const edges = getGraph().filterEdges((e) =>
            getGraph()!
                .extremities(e)
                .some((n) => nodesIndex.has(n)),
        );

        if (!node) {
            state.selectedNode = undefined;
            state.selectedNeighbors = undefined;
        }

        // Refresh rendering
        renderer.refresh({
            partialGraph: {
                nodes,
                edges,
            },
            // We don't touch the graph data so we can skip its reindexation
            skipIndexation: true,
        });
        setGraphState({ ...state });
    }

    onCleanup(() => {
        if (renderer != null) {
            renderer.kill();
        }
    });

    createEffect(() => {
        theme();
        if (renderer) {
            renderer.setSetting("labelColor", {
                attribute: undefined,
                color: theme() == "dark" ? "#ffff" : "#000",
            });
        }
    });

    const min_max_node_size = () => {
        let min = Infinity;
        let max = 0;

        getGraph().forEachNode((node, attrs) => {
            const v = attrs[renderSettings().size_by];

            if (v < min) {
                min = v;
            }

            if (v > max) {
                max = v;
            }
        });

        return [min, max];
    };

    createEffect(() => {
        // console.log("resize");

        loaded();

        const [min, max] = min_max_node_size();
        resizeNodes(getGraph(), (n) =>
            range(
                min,
                max,
                renderSettings().min_size,
                renderSettings().max_size,
                getGraph().getNodeAttribute(n, renderSettings().size_by),
            ),
        );
    });

    onMount(async () => {
        // graph = await fromGraphmlEndpoint("/api/dewiki/graphml");
        if (graphFetch !== undefined) {
            // graph = await graphFetch();
            try {
                const g = await graphFetch();
                setGraph(g);
            } catch (e: any) {
                setLoadingError(e.toString());
            }
        }

        const AMOUNT_MANY_NODES = 200;

        // 2. Render the graph:
        const container = document.getElementById(
            "sigma-container",
        ) as HTMLElement;
        renderer = new Sigma(getGraph(), container, {
            // defaultEdgeColor: "#e6e6e6",
            // labelSize: 15,
            defaultEdgeType:
                getGraph().nodes.length >= AMOUNT_MANY_NODES
                    ? "edges-fast"
                    : "edges-default",
            edgeProgramClasses: {
                "edges-default":
                    getGraph().type == "directed"
                        ? EdgeArrowProgram
                        : EdgeRectangleProgram,
                "edges-fast": EdgeLineProgram,
            },
            allowInvalidContainer: true,
            defaultDrawNodeHover: drawHover,
            labelColor: {
                attribute: undefined,
                color: theme() == "dark" ? "#ffff" : "#000",
            },
        });

        //TODO: add options
        //TODO: render label a bit above / below edge

        // read labels better
        renderer.getCamera().setState({
            angle: 0.2,
        });

        // Bind graph interactions:
        renderer.on("clickNode", ({ node, event }) => {
            if (event.original.ctrlKey && getGraph() !== undefined) {
                const node_label = getGraph().getNodeAttribute(node, "label");
                open(wiki_link(node_label, wiki_name));
                return;
            }

            // if (event.original.altKey && graph !== undefined) {
            //   addNeighboursDFS(node, graph, getNeighbours, 1, 1);
            //   return;
            // }

            if (graphState().selectedNode == node) {
                setHoveredNode(undefined);
            } else {
                setHoveredNode(node);
            }
        });

        renderer.setSetting("nodeReducer", (node, data) => {
            const res: Partial<NodeDisplayData> = { ...data };
            const state = graphState();

            if (
                state.selectedNeighbors &&
                !state.selectedNeighbors.has(node) &&
                state.selectedNode !== node
            ) {
                res.label = "";
                // res.color = "#f6f6f6";
                res.hidden = true;
            }

            if (
                state.selectedClusterID !== undefined &&
                state.selectedClusterID != data.comm
            ) {
                res.hidden = true;
            }
            if (state.searchedNode && state.searchedNode == res.label) {
                res.highlighted = true;
                res.forceLabel = true;
            }

            return res;
        });

        renderer.setSetting("edgeReducer", (edge, data) => {
            const res: Partial<EdgeDisplayData> = { ...data };
            const state = graphState();

            if (
                state.selectedNode &&
                !getGraph()!.hasExtremity(edge, state.selectedNode)
            ) {
                res.hidden = true;
            }

            return res;
        });

        const loader = document.getElementById("sigma-loading") as HTMLElement;
        loader.hidden = true;

        layout = new ForceSupervisor(getGraph());
        if (autoStartLayout) {
            layout.start();
        }

        // const sensibleSettings = forceAtlas2.inferSettings(getGraph());
        // const fa2Layout = new FA2Layout(getGraph(), {
        //   settings: sensibleSettings,
        // });
        // fa2Layout.start

        // const layout = new NoverlapLayout(getGraph());
        // layout.start();

        const temp_clusters = new Map<string, Cluster>();
        getGraph().forEachNode((_node, atts) => {
            if (!temp_clusters.has(atts.comm)) {
                temp_clusters.set(atts.comm, {
                    color: atts.color,
                    id: atts.comm,
                    num_nodes: 1,
                });
            } else {
                const current_nodes = temp_clusters.get(atts.comm)!.num_nodes;
                temp_clusters.set(atts.comm, {
                    ...temp_clusters.get(atts.comm)!,
                    num_nodes: current_nodes + 1,
                });
            }
        });
        setClusters(temp_clusters);
    });

    const toggleLayout = () => {
        if (layout.isRunning()) {
            layout.stop();
            setLayoutIsRunning(false);
        } else {
            layout.start();
            setLayoutIsRunning(true);
        }
    };

    const resetZoom = () => {
        renderer.getCamera().animatedReset({ duration: 600 });
    };

    const refreshRenderer = () => {
        renderer.refresh();
    };

    const searchOnChange = (prefix: string) => {
        setGraphState({ ...graphState(), searchedNode: prefix });
        refreshRenderer();
    };

    const children = (
        <>
            <div class="absolute top-7 left-4">
                <p class="text-sm text-neutral-400">
                    Press{" "}
                    <span class="border py-0.5 px-1 rounded dark-layer-1 bg-white">
                        Ctrl
                    </span>{" "}
                    while clicking to open the wikipedia article
                </p>

                {graphState().selectedNode !== undefined &&
                getGraph() !== undefined ? (
                    <>
                        <p>
                            Selected node:{" "}
                            {getGraph().getNodeAttribute(
                                graphState().selectedNode,
                                "label",
                            )}
                        </p>
                        <p>
                            Showing {graphState().selectedNeighbors?.size}{" "}
                            neighbors
                        </p>
                        <button
                            class="button dark-layer-1 bg-white"
                            onClick={() => setHoveredNode(undefined)}
                        >
                            Reset
                        </button>
                    </>
                ) : (
                    <p class="text-sm text-neutral-400">
                        Select a node and its neighbors by clicking it
                    </p>
                )}
            </div>
        </>
    );

    return (
        <GraphContainer
            containerRef={container}
            children={children}
            controllerProps={{
                toggleLayout: showLayoutToggler && toggleLayout,
                resetZoom,
                layoutIsRunning: layoutIsRunning,
            }}
            settingsProps={{
                wiki_name,
            }}
            clusterProps={
                showClusters
                    ? { graphState, setGraphState, clusters, refreshRenderer }
                    : undefined
            }
            searchOnChange={searchOnChange}
            loadingError={loadingError()}
        />
    );
}

interface ContainerProps {
    containerRef: HTMLDivElement;
    controllerProps: ControllerProps;
    settingsProps: SettingsProps;
    clusterProps?: Omit<ClusterProps, "maxHeight" | "isFullscreen">;
    searchOnChange: (node: string) => void;
    children: any;
    loadingError?: string;
}

function GraphContainer(props: ContainerProps) {
    const SigmaLoader: Component = () => (
        <div
            id="sigma-loading"
            class="absolute left-0 right-0 top-0 bottom-0 m-auto w-fit h-fit"
        >
            <svg
                class="animate-spin -ml-1 mr-3 h-14 w-14"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                ></circle>
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
            </svg>
        </div>
    );

    const [isFullscreen, setIsFullscreen] = createSignal(false);
    const maxHeight = 650;

    function toggleFullScreen() {
        const dom = document.getElementById("sigma-wrapper")!;

        if (document.fullscreenElement !== dom) {
            dom.requestFullscreen();
            setIsFullscreen(true);
        } else {
            if (document.exitFullscreen) {
                document.exitFullscreen();
                setIsFullscreen(false);
            }
        }
    }

    return (
        <>
            <div
                id="sigma-wrapper"
                class="bg-white dark:bg-dark_00 w-full py-2 my-5"
            >
                <div class="relative ">
                    <SigmaLoader />
                    <Show when={props.loadingError !== undefined}>
                        <div class="error absolute right-0 left-0 top-1/3 mx-auto w-fit text-center">
                            <span>Error loading: {props.loadingError}</span>
                        </div>
                    </Show>
                    <div
                        id="sigma-container"
                        class={"w-full my-4 border mx-auto dark:border-dark_01"}
                        style={`height: ${isFullscreen() ? "100vh" : `${maxHeight}px`}`}
                        ref={props.containerRef}
                    />
                    <div class="absolute -top-3 right-0 left-0 mx-auto w-fit">
                        <GraphController
                            {...props.controllerProps}
                            toggleFullScreen={toggleFullScreen}
                        />
                    </div>
                    <GraphPanels
                        isFullscreen={isFullscreen}
                        maxHeight={maxHeight}
                        searchOnChange={props.searchOnChange}
                        settingsProps={props.settingsProps}
                        clusterProps={props.clusterProps}
                    />
                    {props.children}
                </div>
            </div>
        </>
    );
}

interface NeighborGraphProps {
    wiki_name: string;
    page_title: string;
    num_links: number;
    times_linked: number;
}

export function NeighborGraph({
    page_title,
    wiki_name,
    num_links,
    times_linked,
    ...rest
}: NeighborGraphProps) {
    let graph = new Graph();

    // TODO: set neighbors for depth one to all
    let renderSettings: GraphRenderSettings = {
        ...DEFAULT_RENDER_SETTINGS,
        depth: 2,
        neighbors: 10,
    };

    onMount(async () => {
        graph.addNode(page_title, {
            label: page_title,
            size: 10,
            color: "red",
            x: Math.random() * 50,
            y: Math.random() * 20,
            num_links,
            times_linked,
        });
        const getNeighbours = async (name: string) => {
            return await getNeighboursFetch(
                wiki_name,
                name,
                renderSettings.neighbors,
            );
        };

        await addNeighboursDFS(page_title, graph, getNeighbours, 1, {
            max_depth: renderSettings.depth,
        });
    });

    return (
        <>
            {/* <p class="text-sm text-secondary">
        Only showing 10 neighbors 2 links deep currently, sorry!
      </p> */}
            <SigmaProvider graph={graph} renderSettings={renderSettings}>
                <GraphView
                    wiki_name={wiki_name}
                    showLayoutToggler={true}
                    autoStartLayout={true}
                />
            </SigmaProvider>
        </>
    );
}

export function TopGraph(props: { wiki_name: string }) {
    let graph = new Graph();

    // onMount(async () => {
    //   graph = await fromGraphmlEndpoint("/api/dewiki/graphml");

    // });

    return (
        <>
            <SigmaProvider graph={graph}>
                <GraphView
                    wiki_name={props.wiki_name}
                    showClusters={true}
                    graphFetch={() =>
                        fromGraphmlEndpoint(`/api/${props.wiki_name}/graphml`)
                    }
                />
            </SigmaProvider>
        </>
    );
}

// export default NeighborGraph;
