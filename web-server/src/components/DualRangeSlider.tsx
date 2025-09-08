import { createSignal } from "solid-js";

interface Props {
    min: number;
    max: number;
    fromValue?: number;
    toValue?: number;
    onInputFrom?: (value: number) => any;
    onInputTo?: (value: number) => any;
}

export default function DualRangeSlider(props: Props) {
    const [fromValue, setFromValue] = createSignal<number>(
        props.fromValue ?? 0,
    );
    const [toValue, setToValue] = createSignal<number>(props.toValue ?? 0);

    const start_percent = () => (fromValue() / props.max) * 100;
    const end_percent = () => (toValue() / props.max) * 100;
    const length_percent = () => end_percent() - start_percent();

    // createEffect(() => {
    //   if (fromValue() > toValue()) {
    //     console.log("AAA");
    //     setFromValue(to);
    //     return;
    //   }

    // });

    return (
        <div class="dual-range-slider">
            <div class="relative min-h-14">
                <input
                    class="from-slider"
                    id="fromSlider"
                    type="range"
                    value={fromValue()}
                    min={props.min}
                    max={props.max}
                    onInput={(e) => {
                        let v = Number(e.currentTarget.value);
                        if (Number.isNaN(v)) {
                            return;
                        }

                        if (v > toValue()) {
                            e.currentTarget.blur();
                            setFromValue(toValue() - 10);
                            return;
                        }

                        setFromValue(v);
                        if (props.onInputFrom) {
                            props.onInputFrom(v);
                        }
                    }}
                />
                <input
                    class="to-slider"
                    id="toSlider"
                    type="range"
                    value={toValue()}
                    min={props.min}
                    max={props.max}
                    onInput={(e) => {
                        let v = Number(e.currentTarget.value);
                        if (Number.isNaN(v)) {
                            return;
                        }

                        if (v < fromValue()) {
                            e.currentTarget.blur();
                            setToValue(fromValue() + 10);
                            return;
                        }

                        setToValue(v);
                        if (props.onInputTo) {
                            props.onInputTo(v);
                        }
                    }}
                />
                <div class="w-full bg-blue-300 h-[8px] absolute rounded" />
                <div
                    class="bg-blue-600 h-[8px] absolute"
                    style={`left: ${start_percent()}%; width: ${length_percent()}%;`}
                />
            </div>
        </div>
    );
}
