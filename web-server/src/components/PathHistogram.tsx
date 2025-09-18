import { Chart, Colors, Legend, LinearScale, Tooltip } from "chart.js";
import ChartDataLabels from "chartjs-plugin-datalabels";
import { Bar } from "solid-chartjs";
import { createSignal, onMount } from "solid-js";
import { formatNumberUnitPrefix } from "../utils";

const MyChart = ({
    depth_histogram,
}: {
    depth_histogram: {
        [key: string]: number;
    };
}) => {
    /**
     * You must register optional elements before using the chart,
     * otherwise you will have the most primitive UI
     */
    onMount(() => {
        Chart.register(Tooltip, Legend, Colors, LinearScale);
    });

    const [showAll, setShowAll] = createSignal(false);

    const sortedEntries = Object.entries(depth_histogram).sort(
        ([k, _v], [l, _b]) => Number(k) - Number(l),
    );

    const skippedEntries = skip_ones(structuredClone(sortedEntries));
    // const skippedEntries = structuredClone(sortedEntries);

    function skip_ones(entries: [string, number][]) {
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

        const slice_idx = 5 * idx_of_max;
        if (slice_idx < entries.length) {
            const last_element = entries.at(-1)!;
            last_element[0] = ".." + last_element[0];

            entries = entries.slice(0, slice_idx);
            entries[entries.length - 1][0] += "..";
            entries.push(last_element);
        }
        return entries;
    }
    let [chartEntries, setChartEntries] = createSignal(skippedEntries);

    const chartData = () => {
        const labels = chartEntries().map(([k, _]) => k);

        const data_points = chartEntries().map(([_, v]) => v);
        return {
            labels,
            datasets: [
                {
                    label: "Pages reachable",
                    data: data_points,
                },
            ],
        };
    };

    const chartOptions = {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
            datalabels: {
                formatter: function (value: any, _context: any) {
                    return formatNumberUnitPrefix(Number(Math.ceil(value)), 3);
                },
            },
        },
    };

    return (
        <div class="my-2">
            <div class="flex justify-between">
                <h3 class="text-2xl">Depth Histogram</h3>
                <button
                    class="button dark-layer-1"
                    onClick={() => {
                        setShowAll(!showAll());
                        if (showAll()) {
                            setChartEntries(sortedEntries);
                        } else {
                            setChartEntries(skippedEntries);
                        }
                    }}
                >
                    {!showAll() ? "Show all" : "Hide some"}
                </button>
            </div>
            <p class="text-base text-secondary">
                Amount of pages reachable by clicking a link x times (length of
                shortest paths)
            </p>
            <div>
                <Bar
                    data={chartData()}
                    options={chartOptions}
                    width={500}
                    height={500}
                    plugins={[ChartDataLabels]}
                />
            </div>
        </div>
    );
};

export default MyChart;
