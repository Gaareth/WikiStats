import { Chart, Colors, Legend, LinearScale, Tooltip } from "chart.js";
import { createSignal, onMount } from "solid-js";
import { cn } from "../../utils";
import { SimpleChart } from "./SimpleChart";

interface Props {
    dataset: { [key: string]: number };
    label: string;
    title: string;
    description: string;
    chartType: "bar" | "line";
    sliceIdx?: number;
    classNameWrapper?: string;
}

const MyChart = (props: Props) => {
    /**
     * You must register optional elements before using the chart,
     * otherwise you will have the most primitive UI
     */
    onMount(() => {
        Chart.register(Tooltip, Legend, Colors, LinearScale);
    });

    const [showAll, setShowAll] = createSignal(false);

    const sortedEntries = Object.entries(props.dataset).sort(
        ([k, _v], [l, _b]) => Number(k) - Number(l),
    );

    const skippedEntries = props.sliceIdx
        ? skipEntries(structuredClone(sortedEntries), props.sliceIdx)
        : centerAroundMax(structuredClone(sortedEntries));
    // const skippedEntries = structuredClone(sortedEntries);

    function skipEntries(entries: [string, number][], slice_idx: number) {
        if (slice_idx < entries.length) {
            const last_element = entries.at(-1)!;
            last_element[0] = ".." + last_element[0];

            entries = entries.slice(0, slice_idx);
            entries[entries.length - 1][0] += "..";
            entries.push(last_element);
        }
        return entries;
    }

    function centerAroundMax(entries: [string, number][]) {
        // const only_ones_start_idx =
        //   entries.length -
        //   entries
        //     .slice()
        //     .reverse()
        //     .findIndex(([k, v]) => v > 1);

        const idx_of_max = Number(
            entries
                .slice()
                .sort(([_k1, v1], [_k2, v2]) => Number(v2) - Number(v1))[0][0],
        );

        // 2 centers the max idx
        const slice_idx = 2 * idx_of_max;

        return skipEntries(entries, slice_idx);
    }

    let [chartEntries, setChartEntries] = createSignal(skippedEntries);

    const chartLabels = () => chartEntries().map(([k, _]) => k);
    const chartDatasets = () => [
        {
            label: props.label,
            data: chartEntries().map(([_, v]) => v),
        },
    ];

    // const chartData = () => {
    //     const labels = chartEntries().map(([k, _]) => k);

    //     const data_points = chartEntries().map(([_, v]) => v);
    //     return {
    //         labels,
    //         datasets: [
    //             {
    //                 label: props.label,
    //                 data: data_points,
    //             },
    //         ],
    //     };
    // };

    const chartOptions = {
        responsive: true,
        maintainAspectRatio: false,
        // plugins: {
        //     datalabels: {
        //         formatter: function (value: any, _context: any) {
        //             return value.toFixed(3);
        //         },
        //     },
        // },
    };

    return (
        <div class={cn("my-2", props.classNameWrapper)}>
            <div class="flex justify-between">
                <h3 class="text-2xl">{props.title}</h3>
                <button
                    class="button dark-layer-1"
                    onClick={() => {
                        setShowAll(!showAll());
                        if (showAll()) {
                            setChartEntries(sortedEntries);
                        } else {
                            setChartEntries(skippedEntries);
                        }
                    }}>
                    {!showAll() ? "Show all" : "Hide some"}
                </button>
            </div>
            <p class="text-base text-secondary">{props.description}</p>
            <div>
                <SimpleChart
                    labels={chartLabels()}
                    datasets={chartDatasets()}
                    height={500}
                    chartOptions={chartOptions}
                    chartType={props.chartType}
                />
            </div>
        </div>
    );
};

export default MyChart;
