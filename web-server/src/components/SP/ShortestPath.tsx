import {
    Match,
    Show,
    Suspense,
    Switch,
    createSignal,
    lazy,
    onMount,
} from "solid-js";
import {
    ErrorCircleIcon,
    LoadingSpinner,
    MaterialSymbolsArrowCoolDownRounded,
} from "../ClientIcons/Icons";
import { SPList } from "./SPList";
import SPResult from "./SPResult";
import StreamLoader from "./StreamLoader";
import { fetchShortestPathClient, type streamData } from "./stream";
// import { SPGraph } from "./SPGraph";

const SPGraph = lazy(() => import("./SPGraph"));

export interface PathEntry {
    name: string;
    num_links: number;
    times_linked: number;
}

interface Props {
    start: string;
    end: string;
    wiki_name: string;
    ssr_paths?: string[][];
}

export default function ShortestPath(props: Props) {
    // let start_path_entry: PathEntry | undefined = undefined;
    // if (path.length == 1) {
    //   start_path_entry = await path_entry_from_title(start, wiki_name);
    // }
    // const [paths, setPaths] = createSignal<string[][] | undefined>(props.paths);
    const [streamData, setStreamData] = createSignal<streamData>({
        visited: 0,
        elapsed_ms: 0,
    });

    const [error, setError] = createSignal<string>();

    // const [streamDataResource, { refetch }] = createResource(async () =>
    //   fetchShortestPathClient({ ...props, setStreamData })
    // );

    onMount(async () => {
        try {
            await fetchShortestPathClient({ ...props, setStreamData });
            setError(undefined);
        } catch (e: any) {
            console.log("Error", e);

            setError(e.toString());
        }
    });
    // const paths = () => streamDataResource()?.paths;
    // const paths = () => [["Jose", "Internet_Archive", "T"]];
    // const paths = () => streamDataResource.latest?.paths;
    const paths = () => props.ssr_paths || streamData().paths;

    const GraphLoading = () => (
        <div class="border dark:border-dark_01 w-full h-96 py-2 my-5 flex flex-col gap-1 justify-center items-center">
            <span class="block w-8">
                <LoadingSpinner />
            </span>
            <span class="text-base text-secondary">Loading Graph</span>
        </div>
    );

    return (
        <section class="my-2">
            <h2 class="text-2xl flex gap-1 items-center">Shortest path</h2>
            <p class="flex flex-wrap gap-2">
                <span class="text-secondary">from</span>
                <span class="font-bold">{props.start} </span>
                <span class="-rotate-90 block mx-1 w-8 h-8">
                    <MaterialSymbolsArrowCoolDownRounded aria-label="dotted arrow pointing right" />
                </span>
                <span class="text-secondary">to</span>
                <span class="font-bold break-all">{props.end}</span>
            </p>
            {props.start == props.end && (
                <p class="text-sm text-slate-400">
                    You are already there (duh)
                </p>
            )}

            <SPResult streamData={streamData} />

            <SPList
                start={props.start}
                end={props.end}
                wiki_name={props.wiki_name}
                paths={paths()}
            >
                <div class="border min-h-14 flex items-center relative dark:border-dark_05">
                    <div class="-z-10 top-0 w-full h-full absolute animate-pulse bg-slate-100 dark:bg-dark_01  "></div>

                    <div class="mx-auto w-fit py-2 px-2">
                        <Switch
                            fallback={
                                <div class="flex flex-col gap-2 items-center py-2">
                                    <span class="block w-6 h-6">
                                        <LoadingSpinner />
                                    </span>
                                    <p class="text-secondary text-base">
                                        Loading...
                                    </p>
                                </div>
                            }
                        >
                            <Match when={error()}>
                                <div class="flex flex-wrap gap-1 error items-center justify-center">
                                    <span class="block w-8 mx-3">
                                        <ErrorCircleIcon title="error" />
                                    </span>
                                    <div class="text-base">
                                        Failed loading data
                                        <p>{error()}</p>
                                    </div>
                                </div>
                            </Match>
                            <Match when={streamData().elapsed_ms != 0}>
                                <StreamLoader
                                    streamData={streamData}
                                    setStreamData={setStreamData}
                                />
                            </Match>
                        </Switch>
                    </div>
                </div>
            </SPList>

            <Show when={props.ssr_paths === undefined && paths()?.length != 0}>
                <Suspense fallback={<GraphLoading />}>
                    <Show when={paths() != null}>
                        <SPGraph wiki_name={props.wiki_name} paths={paths()!} />
                    </Show>
                </Suspense>
            </Show>
        </section>
    );
}
