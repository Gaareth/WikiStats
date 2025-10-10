import { SimpleChart } from "./Charts/SimpleChart";

function Comp() {
    return (
        <SimpleChart
            labels={["a"]}
            datasets={[{ label: "a", data: [1] }]}
            chartType={"bar"}
            client:only="solid-js"
            height={350}
        />
    );
}
export default Comp;
