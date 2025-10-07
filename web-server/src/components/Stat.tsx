import { createUniqueId, Show } from "solid-js";
import type { Trend } from "../db/constants";
import { cn } from "../utils";
import Card from "./Card";
import { InfoIcon } from "./ClientIcons/Icons";
import TrendPill from "./TrendPill";
import WikiLink from "./WikiLink";

interface Props {
    title: string;
    value?: string | number;
    link?: string;
    wiki_link?: { wiki_name: string; page_title: string };
    className?: string;
    tooltipDescription?: string;
    trend?: Trend;
}

const Stat = (props: Props) => {
    const uid = createUniqueId();

    return (
        <Card className={cn(props.className, "relative px-4 py-6")}>
            <div class="flex flex-wrap gap-1 sm:gap-2 justify-center items-center">
                <p class="text-lg">{props.title}</p>

                <Show when={props.tooltipDescription}>
                    <button
                        data-tooltip-target={`tooltip-stat-${uid}`}
                        aria-describedby={`tooltip-stat-${uid}`}
                        type="button"
                        class="text-secondary">
                        <span class="block w-4 h-4">
                            <InfoIcon />
                        </span>
                    </button>
                    <div
                        class="tooltip default-tooltip z-20"
                        id={`tooltip-stat-${uid}`}
                        role="tooltip">
                        {props.tooltipDescription}
                        <div class="tooltip-arrow" data-popper-arrow />
                    </div>
                </Show>
            </div>

            <Show when={props.wiki_link != null}>
                <div class="flex justify-center">
                    <WikiLink
                        wiki_name={props.wiki_link!.wiki_name}
                        page_name={props.wiki_link!.page_title}
                        class_name="font-bold"
                    />
                </div>
                <span>{props.value}</span>
            </Show>

            <Show when={!props.wiki_link && props.link}>
                <a
                    href={props.link}
                    class="font-bold text-ellipsis overflow-hidden">
                    {props.value}
                </a>
            </Show>

            <Show when={!props.wiki_link && !props.link}>
                <p class="font-bold text-ellipsis overflow-hidden">
                    {props.value}
                </p>
            </Show>

            <Show when={props.trend} keyed>
                {(trend) => (
                    <div class="absolute top-0 right-0">
                        <TrendPill trend={trend} relative={true} />
                    </div>
                )}
            </Show>
        </Card>
    );
};

export default Stat;
