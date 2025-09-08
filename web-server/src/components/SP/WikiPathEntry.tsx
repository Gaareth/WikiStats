import type { JSX } from "solid-js";
import {
    ErrorBoundary,
    Match,
    Show,
    Suspense,
    Switch,
    createResource,
} from "solid-js";
import { WIKIPEDIA_REST_API_HEADERS } from "../../utils";
import {
    StartIcon,
    TargetHitIcon,
    TargetIcon,
    TargetMissedIcon,
} from "../ClientIcons/Icons";
import PageLinkPills from "../PageLinkPills";
import Pill from "../Pill";

interface Props {
    name: string;
    wiki_name: string;
    pills?: JSX.Element;
    type?: "start" | "end" | "end_hit" | "end_miss";
}

export default function WikiPathEntry(props: Props) {
    const wiki_prefix = props.wiki_name.slice(0, 2);
    const wikipedia_url = `https://${wiki_prefix}.wikipedia.org/wiki/${props.name}`;

    const [desc] = createResource(async () => {
        const content = await (
            await fetch(
                `https://${wiki_prefix}.wikipedia.org/api/rest_v1/page/summary/${props.name}`,
                {
                    headers: WIKIPEDIA_REST_API_HEADERS,
                },
            )
        ).json();
        return content["extract"]?.substring(0, 100);
    });

    const check_is_redirect = async () => {
        const base = import.meta.env.SSR ? "http://localhost:4321" : "";

        return (
            await fetch(
                `${base}/api/${props.wiki_name}/${props.name}/is_redirect`,
                {
                    headers: WIKIPEDIA_REST_API_HEADERS,
                },
            )
        ).json();
    };

    const [is_redirect] = createResource(check_is_redirect);

    return (
        <div class="flex flex-col gap-1 px-4 py-2 border dark:bg-dark_01 dark:border-dark_05 max-[450px]:!px-2">
            <div class="flex flex-wrap justify-between gap-1 sm:gap-2">
                <a href={wikipedia_url} class="font-medium break-all">
                    {props.name}
                </a>
                <div class="flex flex-row flex-wrap gap-1 items-center">
                    <Show when={props.type == "start"}>
                        <Pill class="bg-green-100 dark:bg-green-500">
                            <span class="block w-5">
                                <StartIcon
                                    title="start page"
                                    class="text-green-500 dark:text-green-50"
                                />
                            </span>
                        </Pill>
                    </Show>

                    <Suspense>
                        <Show when={is_redirect()}>
                            <Pill class="uppercase">redirect</Pill>
                        </Show>
                    </Suspense>

                    <PageLinkPills
                        title={props.name}
                        wiki_name={props.wiki_name}
                    />

                    {props.pills}

                    <Show
                        when={["end", "end_hit", "end_miss"].includes(
                            props.type!,
                        )}
                    >
                        <Pill class="bg-red-100 dark:bg-red-500">
                            <span class="block w-5">
                                <Switch>
                                    <Match when={props.type == "end"}>
                                        <TargetIcon
                                            title="target page"
                                            class="text-red-500 dark:text-red-50"
                                        />
                                    </Match>
                                    <Match when={props.type == "end_hit"}>
                                        <TargetHitIcon title="target page (hit)" />
                                    </Match>
                                    <Match when={props.type == "end_miss"}>
                                        <TargetMissedIcon title="target page (missed)" />
                                    </Match>
                                </Switch>
                            </span>
                        </Pill>
                    </Show>
                </div>
            </div>
            <ErrorBoundary
                fallback={
                    <p class="error text-base">
                        Failed loading page description:
                        {desc.error}
                    </p>
                }
            >
                <Suspense
                    fallback={
                        <div class="text-base bg-skeleton px-2 h-3 w-20"></div>
                    }
                >
                    <p class="text-base text-neutral-400">{desc()}..</p>
                </Suspense>
            </ErrorBoundary>
        </div>
    );
}
