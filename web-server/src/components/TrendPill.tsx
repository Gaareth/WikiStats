import { Match, Switch } from "solid-js";
import type { Trend } from "../db/constants";
import { big_num_format, cn } from "../utils";
import { TrendingDown, TrendingFlat, TrendingUp } from "./ClientIcons/Icons";
import Pill from "./Pill";

interface Props {
    trend: Trend;
    relative: boolean;
}

export default function TrendPill(props: Props) {
    const colors = {
        up: "text-green-700 bg-green-100 dark:text-green-100 dark:bg-green-600",
        down: "text-red-700 bg-red-100 dark:text-red-100 dark:bg-red-600",
        "no change": "",
    };

    return (
        <Pill
            class={cn(
                "m-1 text-xs flex items-center gap-0",
                colors[props.trend.trend],
            )}>
            <span class="block w-4">
                <Switch fallback={<TrendingFlat />}>
                    <Match when={props.trend?.trend == "up"}>
                        <TrendingUp />
                    </Match>
                    <Match when={props.trend?.trend == "down"}>
                        <TrendingDown />
                    </Match>
                    <Match when={props.trend?.trend == "no change"}>
                        <TrendingFlat />
                    </Match>
                </Switch>
            </span>
            <Switch>
                <Match when={props.relative && props.trend.relValue == 0}>
                    +{big_num_format(props.trend.absValue)}
                </Match>
                <Match when={props.relative}>
                    {big_num_format(props.trend.relValue.toFixed(2))}%
                </Match>
                <Match when={!props.relative}>
                    {big_num_format(props.trend.relValue)}
                </Match>
            </Switch>
        </Pill>
    );
}
