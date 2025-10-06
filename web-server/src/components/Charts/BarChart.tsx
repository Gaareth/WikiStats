import type { ChartOptions } from "chart.js";
import { SimpleChart } from "./SimpleChart"

interface Props {
    title?: string;
    labels: string[];
    datasets: { label: string; data: number[] }[];
    chartOptions?: ChartOptions;
    height?: number;
    [key: string]: any;
}

export const BarChart = (props: Props) => {
    return <SimpleChart {...props} chartType="bar" />
}