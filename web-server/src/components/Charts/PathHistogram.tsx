import SkippedEntriesChart from "./SkippedEntriesChart";

const MyChart = ({
    depth_histogram,
}: {
    depth_histogram: {
        [key: string]: number;
    };
}) => {
    return (
        <SkippedEntriesChart
            dataset={depth_histogram}
            label="% All Pages"
            title="Depth Histogram"
            description="Percentage of all pages by clicking a link x times (length of
                shortest paths)"
            chartType="bar"
        />
    );
};

export default MyChart;
