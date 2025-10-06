import { type ChartOptions } from "chart.js";
import { createSignal, For } from "solid-js";
import { cn } from "../../utils";
import { LineChart } from "./LineChart";
import { SimpleChart } from "./SimpleChart";

export type DatasetType = { label: string; data: number[]; labels: string[] };

interface Props {
    title?: string;
    selectionLabels: string[];
    datasets: DatasetType[][];
    chartOptions?: ChartOptions[];
    chartOptionsAll: ChartOptions;
    numShow?: number;

    chartType: "bar" | "line";
    [key: string]: any;
}

const DEFAULT_SHOWING = 3;

export const DatasetSelection = (props: Props) => {
    const [selection, setSelection] = createSignal(0);

    const opt = () => {
        return props.chartOptions ? props.chartOptions[selection()] : {};
    };


    let selectElement!: HTMLSelectElement;

    return (
        <div>
            <div class="flex flex-wrap justify-between">
                <p class="flex items-center">{props.title}</p>
                <div class="mt-1 sm:mt-0 w-full sm:w-auto">
                    <label class="text-secondary text-sm block">
                        Select dataset:
                    </label>
                    <div class="grid grid-cols-2 sm:flex flex-wrap gap-1">
                        <For
                            each={props.datasets.slice(
                                0,
                                props.numShow ?? DEFAULT_SHOWING,
                            )}>
                            {(_, index) => (
                                <button
                                    role="radio"
                                    onClick={() => {
                                        setSelection(index);
                                        selectElement.selectedIndex = 0;
                                    }}
                                    class={cn(
                                        "button-select",
                                        index() == selection() &&
                                            "button-select-selected",
                                    )}>
                                    {props.selectionLabels[index()]}
                                </button>
                            )}
                        </For>
                        <select
                            name="selection"
                            class={cn(
                                "input-default bg-white hover:bg-neutral-50 !dark-layer-1 col-span-1 appearance-none",
                                selection() >= DEFAULT_SHOWING
                                    ? "button-select-selected"
                                    : "text-secondary",
                            )}
                            onInput={(e) =>
                                setSelection(Number(e.target.value))
                            }
                            ref={selectElement}>
                            <option value="none" selected disabled>
                                more..
                            </option>
                            <For
                                each={props.datasets.slice(
                                    props.numShow ?? DEFAULT_SHOWING,
                                )}>
                                {(_, index) => (
                                    <option
                                        value={
                                            index() +
                                            (props.numShow ?? DEFAULT_SHOWING)
                                        }>
                                        {
                                            props.selectionLabels[
                                                index() +
                                                    (props.numShow ??
                                                        DEFAULT_SHOWING)
                                            ]
                                        }
                                    </option>
                                )}
                            </For>
                        </select>
                    </div>
                </div>
            </div>

            <SimpleChart
                labels={props.datasets[selection()][0].labels}
                datasets={props.datasets[selection()].map(
                    ({ label, data }) => ({
                        label,
                        data,
                    }),
                )}
                client:load
                height={350}
                chartOptions={{ ...props.chartOptionsAll, ...opt() }}
                chartType={props.chartType}
            />
        </div>
    );
};

export default DatasetSelection;
