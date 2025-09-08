// import "solid-js";
// // @ts-ignore
// import { Select, createAsyncOptions } from "@thisbeyond/solid-select";
// import "@thisbeyond/solid-select/style.css";
// import Fuse from "fuse.js";
// import { createSignal, onMount } from "solid-js";
// import type { Pages } from "../pages/api/[wiki_name]/pages";
// import { shuffleArray } from "../utils";

// const fetchData = async (
//   site: string,
//   input_value: any,
//   wiki_name: string | undefined,
//   sp_start: boolean
// ): Promise<string[]> => {
//   if (wiki_name === undefined) {
//     return [];
//   }
//   let json: Pages;
//   // let resp = await fetch(`${site}/api/ja/pages?${input_value}`);
//   // console.log(resp);

//   // json = await fetch(
//   //   `/api/${wiki_name}/pages?prefix=${input_value}&${
//   //     sp_start ? "sp_start=true" : ""
//   //   }`
//   // ).then((resp) => resp.json());

//   try {
//     // const resp = await fetch(
//     //   `/api/${wiki_name}/pages?prefix=${input_value}&${
//     //     sp_start ? "sp_start=true" : ""
//     //   }`
//     // );
//     const resp = await fetch(
//       `/api/${wiki_name}/pages?prefix=${input_value}`
//     );

//     json = await resp.json();
//     // console.log(resp);
//   } catch {
//     json = [];
//   }
//   // json = [{ pageTitle: "a", pageId: 2, wikiName: wiki_name }];
//   // console.log(input_value);
//   // if ((input_value == "" || input_value == null) && !sp_start) {
//   //   return [];
//   // }

//   // const url = `${site}/api/${wiki_name}/pages?prefix=${input_value}&${
//   //   sp_start ? "sp_start=true" : ""
//   // }`;
//   // console.log(url);

//   // const resp = await fetch(url);

//   // json = await resp.json();
//   const page_titles = json.map((p) => p.pageTitle);

//   let result: string[] = page_titles;
//   if (input_value.length > 0) {
//     const fuse = new Fuse(page_titles);

//     result = fuse.search(input_value).map(({ item }) => item);
//   } else {
//     shuffleArray(result);
//   }

//   return result;
// };

// interface Props {
//   name: string;
//   [x: string]: any;
//   wiki_name: string | undefined;
//   current_value: string | null;
// }

// const PageSelect = ({
//   name,
//   site,
//   wiki_name,
//   sp_start = false,
//   current_value,
//   ...restProps
// }: Props) => {
//   // const props = createAsyncOptions(fetchData);
//   // const { name, sp_start, ...restProps } = inputProps;
//   // console.log(selectable_options);
//   // console.log(restProps);
//   const [value, setValue] = createSignal(current_value ?? "");
//   const [props, setProps] = createSignal(
//     createAsyncOptions((input: any) =>
//       fetchData(site, input, undefined, sp_start)
//     )
//   );

//   // let props = createAsyncOptions((input: any) =>
//   //   fetchData(site, input, "ja", sp_start)
//   // );

//   onMount(() => {
//     // SSR Load does not fetch links
//     // ON mount recreate options to fetch links
//     // :^)
//     setProps(
//       createAsyncOptions(
//         (input: any) => fetchData(site, input, wiki_name, sp_start),
//         { creatable: true }
//       )
//     );
//   });

//   return (
//     <>
//       <Select
//         class="solid-select !px-0 w-full"
//         loading={true}
//         initialValue={current_value}
//         {...props}
//         {...restProps}
//         onChange={(v: any) => setValue(v?.toString())}
//         isOptionDisabled={(option: string) => option === value()}
//         disabled={wiki_name === undefined}
//       />
//       <input type="hidden" name={name} value={value()} />
//     </>
//   );
// };

// export default PageSelect;
