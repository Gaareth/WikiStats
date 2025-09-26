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
import { Bar } from "solid-chartjs";
import { createSignal, onMount, splitProps } from "solid-js";
import { cn } from "../../utils";
import { LoadingSpinner } from "../ClientIcons/Icons";

interface Props {
    title?: string;
    labels: string[];
    datasets: { label: string; data: number[] }[];
    chartOptions?: ChartOptions;
    height?: number;
    [key: string]: any;
}

export const BarChart = (props: Props) => {
    /**
     * You must register optional elements before using the chart,
     * otherwise you will have the most primitive UI
     */
    onMount(() => {
        Chart.register(Title, Tooltip, Legend, Colors, LogarithmicScale);
    });
    const [local, rest] = splitProps(props, ["labels", "datasets", "title"]);

    const chartData = (): ChartData => ({
        labels: local.labels,
        datasets: local.datasets,
    });

    const plugins = () => ({
        title: {
            display: true,
            text: local.title,
        },
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
                <span class="block w-12 absolute left-0 right-0 top-0 bottom-0 m-auto h-fit">
                    <LoadingSpinner />
                </span>
                <span class="text-secondary">Loading Chart..</span>
            </div>

            <div class="w-full h-full" hidden ref={wrapperRef}>
                <Bar data={chartData()} options={chartOptions()} {...rest} />
            </div>
        </div>
    );
};
