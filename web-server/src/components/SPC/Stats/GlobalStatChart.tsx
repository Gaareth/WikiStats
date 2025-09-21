import { createSignal, For } from "solid-js";
import { BarChart } from "../../Charts/BarChart";

interface Props {
    datasets: { label: string; data: number[] }[];
    labels: string[];
}

export const Chart = (props: Props) => {
    const [selection, setSelection] = createSignal(0);

    return (
        <div>
            <div>
                <For each={props.datasets}>
                    {(_, index) => (
                        <button
                            onClick={() => setSelection(index)}
                            class="button">
                            {index()}
                        </button>
                    )}
                </For>
            </div>

            <BarChart
                labels={props.labels}
                datasets={[props.datasets[selection()]]}
                title={String(selection())}
                client:load
                height={350}
            />
        </div>
    );
};

export default Chart;
