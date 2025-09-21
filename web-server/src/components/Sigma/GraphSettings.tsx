import { For, Show, type JSX } from "solid-js";
import { addNeighboursDFS, getNeighboursFetch } from "../../client/graph";
import DualRangeSlider from "../DualRangeSlider";
import GraphPanel from "./GraphPanel";
import { useSigma } from "./SigmaContext";
import { SizeOption } from "./SigmaGraph";

export interface SettingsProps {
    // root_node: string;
    wiki_name: string;
    children?: JSX.Element;
}

function SettingsNeighbors(props: { wiki_name: string }) {
    const { getGraph, setGraph } = useSigma()!["graph"];
    const { renderSettings, setRenderSettings } = useSigma()!["renderSettings"];

    const addNeighbors = async () => {
        const root_node = getGraph().nodes()[0];

        const getNeighbours = async (name: string) => {
            return await getNeighboursFetch(
                props.wiki_name,
                name,
                renderSettings().neighbors,
            );
        };

        await addNeighboursDFS(root_node, getGraph(), getNeighbours, 1, {
            max_depth: renderSettings().depth,
        });
    };

    return (
        <div class="flex gap-2 w-full">
            <label for="neighbors-input" class="w-1/2">
                max neighbors
                <input
                    name="neighbors-input"
                    class="dark-layer-2 input-default w-full"
                    type="number"
                    value={renderSettings().neighbors}
                    onChange={(e) => {
                        const val = Number(e.target.value);
                        if (
                            Number.isNaN(val) ||
                            val == renderSettings().neighbors
                        ) {
                            return;
                        }

                        if (val < renderSettings().neighbors) {
                            const root_node = getGraph().nodes()[0];
                            const attrs =
                                getGraph().getNodeAttributes(root_node);

                            getGraph().clear();
                            getGraph().addNode(root_node, { ...attrs });
                        }
                        setRenderSettings({
                            ...renderSettings(),
                            neighbors: val,
                        });
                        addNeighbors();
                    }}
                />
            </label>

            <label for="depth-input" class="w-1/2">
                depth neighbors
                <input
                    name="depth-input"
                    class="dark-layer-2 input-default w-full"
                    type="number"
                    value={renderSettings().depth}
                    onChange={(e) => {
                        const val = Number(e.target.value);
                        if (
                            Number.isNaN(val) ||
                            val == renderSettings().depth
                        ) {
                            return;
                        }

                        const root_node = getGraph().nodes()[0];
                        const attrs = getGraph().getNodeAttributes(root_node);

                        getGraph().clear();
                        getGraph().addNode(root_node, { ...attrs });

                        setRenderSettings({ ...renderSettings(), depth: val });
                        addNeighbors();
                    }}
                />
            </label>
        </div>
    );
}

export default function GraphSettings(props: SettingsProps) {
    const { renderSettings, setRenderSettings } = useSigma()!["renderSettings"];

    const onSizeByChange = (e: { target: { value: any } }) => {
        const val = e.target.value;

        if (Object.values(SizeOption).includes(val)) {
            const size: SizeOption = val;
            setRenderSettings({ ...renderSettings(), size_by: size });
        }
    };

    return (
        <GraphPanel title="Settings">
            <div class="flex flex-col gap-4">
                <Show
                    when={
                        Object.entries(renderSettings().possible_size_options)
                            .length > 0
                    }
                    fallback={
                        <p class="text-secondary text-sm">
                            Graph contains no properties
                        </p>
                    }>
                    <div class="flex gap-2 justify-between">
                        <label for="size">Size by</label>
                        <select
                            id="size"
                            name="size"
                            class="select dark:!bg-dark_03 dark:!border-dark_04"
                            onChange={onSizeByChange}>
                            <For
                                each={Object.entries(
                                    renderSettings().possible_size_options,
                                )}>
                                {([k, v]) => (
                                    <option
                                        value={k}
                                        selected={
                                            renderSettings().size_by == k
                                        }>
                                        {v}
                                    </option>
                                )}
                            </For>
                        </select>
                    </div>
                </Show>

                <div>
                    <div class="flex gap-1 flex-wrap justify-between mb-2">
                        <p> Min {renderSettings().min_size}</p>
                        <p> Max {renderSettings().max_size}</p>
                    </div>
                    <DualRangeSlider
                        min={1}
                        max={100}
                        fromValue={renderSettings().min_size}
                        toValue={renderSettings().max_size}
                        onInputFrom={(v) =>
                            setRenderSettings({
                                ...renderSettings(),
                                min_size: v,
                            })
                        }
                        onInputTo={(v) =>
                            setRenderSettings({
                                ...renderSettings(),
                                max_size: v,
                            })
                        }
                    />
                </div>

                {props.children}
            </div>

            {/* <SettingsNeighbors wiki_name={props.wiki_name} /> */}
        </GraphPanel>
    );
}
