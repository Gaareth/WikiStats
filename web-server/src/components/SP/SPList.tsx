import { For, Match, Show, Switch } from "solid-js";
import { PathLink } from "./PathLink";
import WikiPathEntry from "./WikiPathEntry";

export interface PathEntry {
    name: string;
    num_links: number;
    times_linked: number;
}

interface Props {
    start: string;
    end: string;
    paths?: string[][];
    wiki_name: string;
    children?: any;
}

export function SPList(props: Props) {
    // let start_path_entry: PathEntry | undefined = undefined;
    // if (props.path.length == 1) {
    //   start_path_entry = await path_entry_from_title(start, wiki_name);
    // }
    // const [path, setPath] = createSignal();

    const unconnected = () => props.paths?.length == 0;
    const end_type = () => {
        if (unconnected()) {
            return "end_miss";
        } else if (props.paths?.length! > 0) {
            return "end_hit";
        }
        return "end";
    };

    const shown_path = () => props.paths?.at(0);

    return (
        <div class="flex flex-col gap-2 mt-2">
            <div class="relative">
                <WikiPathEntry
                    name={props.start}
                    wiki_name={props.wiki_name}
                    type="start"
                />
                <Show
                    when={
                        shown_path() !== undefined && shown_path()?.length != 2
                    }
                >
                    <PathLink
                        className="absolute -bottom-5 justify-center w-full "
                        current_page_title={props.start}
                        next_page_title={shown_path()![1]}
                    />
                </Show>
            </div>

            <Show when={props.paths != null} fallback={props.children}>
                <Switch>
                    <Match when={props.paths?.length == 0}>
                        <div class="my-2 text-red-800 dark:text-red-400 text-center">
                            No connection found
                        </div>
                    </Match>
                    <Match when={props.paths?.length! > 0}>
                        <For each={shown_path()?.slice(1, -1)}>
                            {(path_entry, i) => (
                                <div class="relative">
                                    <WikiPathEntry
                                        name={path_entry}
                                        wiki_name={props.wiki_name}
                                    />
                                    <PathLink
                                        className="absolute -bottom-5 justify-center w-full "
                                        current_page_title={path_entry}
                                        next_page_title={shown_path()![i() + 2]}
                                    />
                                </div>
                            )}
                        </For>
                    </Match>
                </Switch>
            </Show>
            <WikiPathEntry
                name={props.end}
                wiki_name={props.wiki_name}
                type={end_type()}
            />
        </div>
    );
}

// {
//   () => {
//     if (path.length > 0) {
//       if (path.length == 1 && start_path_entry !== undefined) {
//         return (
//           <div class="flex flex-col gap-2 mt-2">
//             <WikiPathEntry {...start_path_entry} {wiki_name} />
//             <div class="my-2 text-red-800 dark:text-red-400 text-center">
//               No connection found
//             </div>
//             <WikiPathEntry {...path[0]} {wiki_name} />
//           </div>
//         );
//       }

//       return (
//         <div class="flex flex-col gap-2 mt-2">
//           {path.map((path_entry) => (
//             <WikiPathEntry {...path_entry} {wiki_name} />
//           ))}
//         </div>
//       );
//     }
//   }
// }
