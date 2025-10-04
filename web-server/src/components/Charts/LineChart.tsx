import {
    Chart,
    Colors,
    Legend,
    Title,
    Tooltip,
    type ChartOptions,
} from "chart.js";
import { Line } from "solid-chartjs";
import { onMount, splitProps } from "solid-js";

interface Props {
    title: string;
    labels: string[];
    datasets: { label: string; data: number[] }[];
    [key: string]: any;
}

export const LineChart = (props: Props) => {
    /**
     * You must register optional elements before using the chart,
     * otherwise you will have the most primitive UI
     */
    onMount(() => {
        Chart.register(Title, Tooltip, Legend, Colors);
    });
    const [local, rest] = splitProps(props, ["labels", "datasets", "title"]);

    console.log(Object.fromEntries(Object.entries(rest)));

    const chartData = {
        labels: local.labels,
        datasets: local.datasets,
    };

    const plugins = {
        title: {
            display: true,
            text: local.title,
        },
    };

    const chartOptions: ChartOptions<"line"> = {
        responsive: true,
        maintainAspectRatio: false,
        plugins: plugins,
    };
    // fix the rest inclusion
    return (
        <>
            <div>
                <Line data={chartData} options={chartOptions} />
            </div>
        </>
    );
};
