import type { ChartOptions } from "chart.js";
import { SimpleChart } from "./SimpleChart";

interface Props {
    // data: { label: string; data: number[] }; // this does not work for some reason, the chart just goes black???
    labels: string[]; // text on the x axis
    data: number[];
    infoValues?: string[] | null;
    label: string;
    chartOptions?: ChartOptions;
    // chartType: "bar" | "line";
}

export const SimpleChartWrapper = (props: Props) => {
    const plugins = {
        tooltip: {
            callbacks: {
                label: function (context: any) {
                    const info = props.infoValues != null ? props.infoValues[context.dataIndex] : undefined;
                    console.log(info);
                    
                    //   const splitInfo = info.match(/.{1,25}/g) || [info];
                    const splitInfo = info?.split("_") ?? []; 
                    return [
                        `${context.dataset.label}: ${context.formattedValue}`, // first line
                        ...splitInfo, // second line
                    ];
                },
            },
        },
    };

    return (
        <SimpleChart
            labels={props.labels}
            datasets={[{ label: props.label, data: props.data }]}
            chartOptions={props.chartOptions}
            chartType={"bar"}
            plugins={plugins}
        />
    );
};
