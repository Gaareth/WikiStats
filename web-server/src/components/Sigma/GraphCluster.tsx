import type { Accessor, Component, Setter } from "solid-js";
import { TooltipButton } from "../TooltipButton";
import GraphPanel from "./GraphPanel";
import type { Cluster, State } from "./SigmaGraph";

export interface ClusterProps {
    clusters: Accessor<Map<string, Cluster>>;
    graphState: Accessor<State>;
    setGraphState: Setter<State>;
    refreshRenderer: () => void;
    maxHeight: number;
    isFullscreen: Accessor<boolean>;
}

export default function GraphCluster({
    clusters,
    graphState,
    setGraphState,
    refreshRenderer,
    maxHeight,
    isFullscreen,
}: ClusterProps) {
    const selectCluster = (cluster: Cluster | undefined) => {
        let selectedClusterID = undefined;

        if (cluster !== undefined) {
            selectedClusterID =
                graphState().selectedClusterID != cluster.id
                    ? cluster.id
                    : undefined;
        }

        setGraphState({
            ...graphState(),
            selectedClusterID,
        });
        // renderer.refresh({
        //   skipIndexation: true,
        // });
        refreshRenderer();
    };

    const ResetClusterButton: Component = () => (
        <TooltipButton
            disabled={graphState().selectedClusterID === undefined}
            onClick={() => selectCluster(undefined)}
            class="button text-sm px-2 group relative dark-layer-2"
            tooltip="Reset selection"
        >
            <p class="hidden">
                Attribution: pajamas:redo
                https://api.iconify.design/pajamas:redo.svg
            </p>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                viewBox="0 0 16 16"
            >
                <path
                    fill="currentColor"
                    fill-rule="evenodd"
                    d="M10.095.28A8 8 0 0 0 1.5 3.335V1.75a.75.75 0 0 0-1.5 0V6h4.25a.75.75 0 1 0 0-1.5H2.523a6.5 6.5 0 1 1-.526 5.994a.75.75 0 0 0-1.385.575A8 8 0 1 0 10.095.279Z"
                    clip-rule="evenodd"
                />
            </svg>
        </TooltipButton>
    );

    return (
        <GraphPanel title="Communities">
            <div class="flex justify-between gap-2 my-2">
                <p>{clusters().size} Communities</p>
                <ResetClusterButton />
            </div>
            <div
                class={`h-full flex flex-col gap-1 overflow-scroll  `}
                style={`max-height: ${
                    isFullscreen() ? "calc(100vh - 200px);" : `${maxHeight}px`
                }`}
            >
                {Array.from(clusters())
                    .sort(([, c1], [, c2]) => c2.num_nodes - c1.num_nodes)
                    .map(([, cluster]) => (
                        <button
                            class={`button flex gap-2 items-center h-full justify-end px-3 py-2 border dark-layer-2
                    ${
                        graphState().selectedClusterID == cluster.id &&
                        "!border-blue-500"
                    } hover:border-blue-300`}
                            onClick={() => selectCluster(cluster)}
                        >
                            <span>#{cluster.id}</span>
                            <span>{cluster.num_nodes}</span>
                            <div
                                class={`rounded-full w-4 h-4`}
                                style={`background-color:${cluster.color}`}
                            ></div>
                        </button>
                    ))}
            </div>
        </GraphPanel>
    );
}
