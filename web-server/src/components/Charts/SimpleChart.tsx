import {
    Chart,
    Colors,
    Legend,
    LogarithmicScale,
    Title,
    Tooltip,
    type ChartData,
    type ChartOptions,
} from "chart.js";
import { Bar, Line } from "solid-chartjs";
import { createSignal, Match, onMount, splitProps, Switch } from "solid-js";
import { cn } from "../../utils";
import { LoadingSpinner } from "../ClientIcons/Icons";

export interface SimpleChartProps {
    title?: string;
    labels: string[]; // text on the x axis
    datasets: { label: string; data: number[] }[];
    chartOptions?: ChartOptions;
    height?: number;
    [key: string]: any;
    chartType: "bar" | "line";
}

export const SimpleChart = (props: SimpleChartProps) => {
    /**
     * You must register optional elements before using the chart,
     * otherwise you will have the most primitive UI
     */
    onMount(() => {
        Chart.register(Title, Tooltip, Legend, Colors, LogarithmicScale);
    });
    const [local, rest] = splitProps(props, [
        "labels",
        "datasets",
        "title",
        "chartType",
        
    ]);


    const chartData = (): ChartData => ({
        labels: local.labels,
        datasets: local.datasets,
    });

    const plugins = () => ({
        title: {
            display: true,
            text: local.title,
        },
        ...rest.plugins
    });

    const chartOptions = (): ChartOptions => ({
        responsive: true,
        maintainAspectRatio: false,
        plugins: plugins(),
        ...rest.chartOptions,
    });

    // there is weird "flickering"? on mobile ios? devices, otherwise
    // It looks like the chart overflows and then shrinks to the correct size
    // this fixes it
    const [loaded, setLoaded] = createSignal(false);
    let wrapperRef!: HTMLDivElement;

    onMount(() => {
        wrapperRef.hidden = false;
        setLoaded(true);
    });

    return (
        <div class="relative" style={`min-height: ${rest.height}px`}>
            <div class={cn(loaded() && "hidden")}>
                <div class="absolute left-0 right-0 top-0 bottom-0 m-auto h-fit flex flex-col items-center">
                    <span class="block w-12">
                        <LoadingSpinner />
                    </span>
                    <span class="text-secondary">Loading Chart..</span>
                    <noscript>
                        <span class="text-error">
                            Enable Javascript to load this chart
                        </span>
                    </noscript>
                </div>
            </div>

            <div class="w-full h-full" hidden ref={wrapperRef}>
                <Switch>
                    <Match when={local.chartType == "bar"}>
                        <Bar
                            data={chartData()}
                            options={chartOptions()}
                            {...rest}
                        />
                    </Match>
                    <Match when={local.chartType == "line"}>
                        <Line
                            data={chartData()}
                            options={chartOptions()}
                            {...rest}
                        />
                    </Match>
                </Switch>
            </div>
        </div>
    );
};
