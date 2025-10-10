import { type JSX, Show } from "solid-js";
import type { Trend } from "../db/constants";
import { cn } from "../utils";
import Card from "./Card";
import { InfoIcon } from "./ClientIcons/Icons";
import { TooltipButton } from "./TooltipButton";
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
    trend_link?: string;
    children?: JSX.Element;
}

const Stat = (props: Props) => {
    return (
        <Card
            className={cn(
                props.className,
                "relative px-4 py-6",
                props.trend_link &&
                    "text-current hover:bg-gray-50 !no-underline dark:hover:bg-dark_02 dark:hover:border-dark_03",
            )}
            wrapperComponent={
                props.trend_link && !props.wiki_link ? "a" : "div"
            }
            href={props.trend_link}>
            <div class="flex flex-wrap gap-1 sm:gap-2 justify-center items-center">
                <p class="text-lg">{props.title}</p>

                <Show when={props.tooltipDescription}>
                    <TooltipButton
                        tooltip={props.tooltipDescription ?? ""}
                        type="button"
                        class="text-secondary">
                        <span class="block w-4 h-4">
                            <InfoIcon />
                        </span>
                    </TooltipButton>
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

            {props.children}
        </Card>
    );
};

export default Stat;
