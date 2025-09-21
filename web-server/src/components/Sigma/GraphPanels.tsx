import { Show, createSignal, type Accessor } from "solid-js";
import { cn } from "../../utils";
import GraphCluster, { type ClusterProps } from "./GraphCluster";
import GraphSearch from "./GraphSearch";
import GraphSettings, { type SettingsProps } from "./GraphSettings";

interface Props {
    isFullscreen: Accessor<boolean>;
    settingsProps: SettingsProps;
    clusterProps?: Omit<ClusterProps, "maxHeight" | "isFullscreen">;
    searchOnChange: (node: string) => void;
    maxHeight: number;
}

export default function GraphPanels(props: Props) {
    const [expanded, setExpanded] = createSignal(false);

    const heightOfOtherPanels = 250;

    return (
        <div
            class={`absolute right-2 ${
                props.isFullscreen() ? " bottom-8" : "bottom-2"
            }`}>
            <div class="flex justify-end m-2">
                <button
                    class="w-10 h-10 rounded-full bg-white
                        dark-layer-1 border hover:bg-gray-10 
                        flex items-center justify-center"
                    onClick={() => setExpanded(!expanded())}
                    aria-label={(expanded() ? "Close" : "Open") + " panels"}>
                    <div
                        class={cn(
                            "tham tham-e-squeeze tham-w-6",
                            expanded() && "tham-active",
                        )}>
                        <div class="tham-box">
                            <div class="tham-inner bg-gray-500 dark:bg-gray-400" />
                        </div>
                    </div>
                </button>
            </div>

            <Show when={expanded()}>
                <div class="w-80 flex flex-col gap-2">
                    <GraphSearch onChange={props.searchOnChange} />
                    <GraphSettings {...props.settingsProps} />
                    {props.clusterProps && (
                        <GraphCluster
                            {...props.clusterProps}
                            maxHeight={props.maxHeight - heightOfOtherPanels}
                            isFullscreen={props.isFullscreen}
                        />
                    )}
                </div>
            </Show>
        </div>
    );
}
