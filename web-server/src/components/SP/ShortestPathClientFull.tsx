// import {
//   Show,
//   createResource,
//   createSignal,
//   type Setter,
//   onMount,
//   createEffect,
//   Switch,
//   Match,
//   type Accessor,
// } from "solid-js";
// import { big_num_format } from "../../utils";
// import { LoadingSpinner } from "../ClientIcons/Icons";

// interface FetchProps {
//   start: string;
//   end: string;
//   wiki_name: string;
//   setPaths: Setter<string[][] | undefined>;
// }

// interface Props extends FetchProps {
//   max_pages?: number;
//   full?: boolean;
//   streamData: Accessor<streamData>;
//   setStreamData: Setter<streamData>;
// }

// const Path = (props: Props) => {
//   // const [perSec, setPerSec] = createSignal(1);

//   const p = {
//     start: props.start,
//     end: props.end,
//     wiki_name: props.wiki_name,
//     setPaths: props.setPaths,
//   };
//   const [pathResource, { refetch }] = createResource(p, fetchPath);

//   onMount(() => {
//     refetch();
//   });

//   createEffect(() => {
//     const paths = pathResource()?.paths;
//     if (!pathResource.loading && paths !== null) {
//       props.setPaths(paths);
//     }
//   });
//   return (
//     <>
//       <Show
//         when={
//           pathResource()?.paths != null && pathResource()?.paths?.length! > 0
//         }
//       >
//         <p class="mt-2">
//           Found a{" "}
//           <span class="font-bold">{pathResource()!.paths![0]?.length}</span>{" "}
//           pages long path in{" "}
//           <span class="font-bold">{pathResource()?.elapsed_ms}ms</span> visiting{" "}
//           <span class="font-bold">{pathResource()?.visited}</span> pages
//         </p>

//         <Switch>
//           <Match when={pathResource()?.visited == 1}>
//             <p>Easy, almost like you started at the target</p>
//           </Match>
//           <Match when={pathResource()?.visited! >= 100}>
//             <p>You have discovered a lot of pages</p>
//           </Match>
//         </Switch>

//         <Switch>
//           <Match when={pathResource()?.paths![0].length! < 5}>
//             <p>That's a short path</p>
//           </Match>
//           <Match when={pathResource()?.paths![0].length! == 5}>
//             <p>Most of the pages are found in 5 steps</p>
//           </Match>
//           {/* <Match when={pathResource()?.paths![0].length! == 5}>
//             <p>Most of the pages are found in 5 steps</p>
//           </Match> */}
//         </Switch>
//       </Show>

//       {/* <progress max={props.max_pages} value={visited()}></progress> */}
//       {/* <p>max time: {prettyMilliseconds((props.max_pages / perSec()) * 1000)}</p> */}

//       {/* <span>
//         {pathResource.loading
//           ? visited() == 0 && "Connecting..."
//           : "Loading..."}
//       </span> */}

//       <span>{pathResource.loading && "Loading..."}</span>
//       <p>{pathResource.error}</p>
//       <Show when={pathResource.error}>
//         <div class="text-red-400">
//           Failed loading data
//           <p>{pathResource.error.toString()}</p>
//         </div>
//       </Show>

//       {/* <Show
//         when={
//           !pathResource.loading &&
//           !pathResource.error &&
//           pathResource() !== undefined
//         }
//       >
//         <div class="flex flex-col gap-2 mt-2">
//           {(pathResource() ?? { shortest_path: [] }).shortest_path.map(
//             (path_entry) => (
//               <p>{path_entry}</p>
//             )
//           )}
//         </div>
//       </Show> */}
//     </>
//   );
// };

// export default Path;
