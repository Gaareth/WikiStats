import clsx, { type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { WIKI_TYPES } from "./constants";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function wiki_link(title: string, lang: string) {
    // const base_url = siteinfo_map.get(lang)?.url;
    // if (base_url == null) {
    //     throw new Error("Invalid wiki name '" + lang + "'. No siteinfo found.");
    // }
    // return `${base_url}/wiki/${title}`;

    const lang_prefix = get_wiki_prefix(lang);
    return `https://${lang_prefix}.wikipedia.org/wiki/${title}`;
}

export function get_wiki_prefix(lang: string) {
    for (const type of WIKI_TYPES) {
        if (lang.endsWith(type)) {
            return lang.slice(0, lang.length - type.length);
        }
    }
    throw new Error(
        "Invalid wiki name '" +
            lang +
            "'. Does not contain any of " +
            WIKI_TYPES.join(", "),
    );
}

export function reverse_string(s: string): string {
    return s.split("").reverse().join("");
}

// formats big numbers using the provided symbol every 3rd digit from the left to improve readability
// 10000 => 10.000 (for symbol=".")
export function big_num_format(
    num: number | string,
    symbol: string | undefined = "_",
): string {
    if (num === undefined) {
        return "undefined";
    }

    num = Number(num);

    const [s, after_decimal] = num.toString().split(".");
    let formatted_string = "";
    let c = 0;

    for (const char of reverse_string(s)) {
        formatted_string += char;
        c += 1;
        if (c % 3 == 0 && c < s.length - (s.startsWith("-") ? 1 : 0)) {
            formatted_string += symbol;
        }
    }

    formatted_string = reverse_string(formatted_string);

    if (after_decimal != null) {
        formatted_string = formatted_string + "." + after_decimal.toString();
    }

    return formatted_string;
}

export function sleep(time: number) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

export function shuffleArray(array: unknown[]) {
    for (let i = array.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [array[i], array[j]] = [array[j], array[i]];
    }
}

export const range = (
    source_min: number,
    source_max: number,
    target_min: number,
    target_max: number,
    a: number,
) => lerp(target_min, target_max, invlerp(source_min, source_max, a));

export const lerp = (x: number, y: number, a: number) => x * (1 - a) + y * a;
export const invlerp = (x: number, y: number, a: number) =>
    clamp((a - x) / (y - x), 0, 1);

export function clamp(value: number, min: number, max: number) {
    if (Number.isNaN(value) || value == null) {
        return min;
    }
    return Math.max(Math.min(value, max), min);
}

// export async function get_precalced_path(start: string, end: string, wiki_name: string) {
//   let path: PathEntry[] = [];
//   let start_id = undefined;
//   let end_id = undefined;

//   if (start.length > 0 && end.length > 0 && wiki_name !== undefined) {
//     start_id = await name_to_page_id(start, wiki_name);
//     end_id = await name_to_page_id(end, wiki_name);

//     if (start_id !== undefined && end_id !== undefined) {
//       // path = await path_from_titles(
//       //   await breadth_first_search(start_id, end_id, wiki_name),
//       //   wiki_name
//       // );
//       if (
//         (await get_supported_pages("", wiki_name)).find(
//           ({ pageTitle }) => pageTitle == start
//         )
//       ) {
//         path = await get_precalced_shortest_path(
//           start_id,
//           end,
//           end_id,
//           wiki_name!
//         );
//       } else {
//         console.log("no precalc");
//       }

//       console.log(path);
//     }
//   }
//   return path;
// }

const ranges = [
    { divider: 1e18, suffix: "E" },
    { divider: 1e15, suffix: "P" },
    { divider: 1e12, suffix: "T" },
    { divider: 1e9, suffix: "G" },
    { divider: 1e6, suffix: "M" },
    { divider: 1e3, suffix: "k" },
];
// https://stackoverflow.com/questions/17633462/format-a-javascript-number-with-a-metric-prefix-like-1-5k-1m-1g-etc
export function formatNumberUnitPrefix(n: number, prec?: number) {
    for (var i = 0; i < ranges.length; i++) {
        if (n >= ranges[i].divider) {
            return (n / ranges[i].divider).toPrecision(prec) + ranges[i].suffix;
        }
    }

    return n.toString();
}

export function formatBytesIntl(
    bytes: number,
    decimals = 2,
    locale: Intl.LocalesArgument = "en-US",
) {
    if (bytes === 0) return "0 Bytes";

    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    const value = bytes / Math.pow(k, i);

    return (
        new Intl.NumberFormat(locale, {
            minimumFractionDigits: 0,
            maximumFractionDigits: decimals,
        }).format(value) +
        " " +
        sizes[i]
    );
}

// https://www.media-division.com/easy-human-readable-date-difference/
// returns array containing the difference and the time unit of measure
export function timeDiff(date1: Date, date2: Date) {
    const timeIntervals = [31536000, 2628000, 604800, 86400, 3600, 60, 1];
    const intervalNames = [
        "year",
        "month",
        "week",
        "day",
        "hour",
        "minute",
        "second",
    ];
    const diff = Math.abs(date2.getTime() - date1.getTime()) / 1000;
    const index = timeIntervals.findIndex((i) => diff / i >= 1);
    const n: number = Math.floor(diff / timeIntervals[index]);
    const interval = intervalNames[index];
    return {
        value: n,
        interval: interval,
    };
}

export function formatTimeDiff(d1: Date, d2: Date) {
    let { value, interval } = timeDiff(d1, d2);
    if (
        Number.isNaN(value) &&
        Math.abs(d1.getMilliseconds() - d2.getMilliseconds()) < 1000
    ) {
        return `less than 1 second`;
    }

    if (value > 1) {
        interval += "s";
    }
    return `${value} ${interval}`;
}

export async function reinitializeFlowBiteTooltips(tooltipRef: HTMLElement) {
    const { Tooltip } = await import("flowbite");

    const tooltipTrigger = tooltipRef;

    if (tooltipTrigger) {
        const tooltipId = tooltipTrigger.dataset.tooltipTarget;
        const tooltipElement = document.getElementById(tooltipId!);

        if (tooltipElement) {
            // Initialize the tooltip only if it hasn't been initialized already
            if (!tooltipElement.hasAttribute("data-tooltip-initialized")) {
                new Tooltip(tooltipElement, tooltipTrigger); // Adjust to Flowbite API
                tooltipElement.setAttribute("data-tooltip-initialized", "true");
            }
        }
    }
}

export function parseDumpDate(dateString: string) {
    const year = parseInt(dateString.slice(0, 4), 10);
    const month = parseInt(dateString.slice(4, 6), 10) - 1; // Months are zero-based in JS Date
    const day = parseInt(dateString.slice(6, 8), 10);

    return new Date(year, month, day);
}

export function formatDumpDate(dateString: string) {
    const year = parseInt(dateString.slice(0, 4), 10);
    const month = parseInt(dateString.slice(4, 6), 10) - 1; // Months are zero-based in JS Date
    const day = parseInt(dateString.slice(6, 8), 10);

    const paddedMonth = String(month + 1).padStart(2, "0");
    const paddedDay = String(day).padStart(2, "0");
    return `${year}-${paddedMonth}-${paddedDay}`;
}

export type KeysOfType<T, ValueType> = Exclude<
    {
        [K in keyof T]: T[K] extends ValueType ? K : never;
    }[keyof T],
    undefined
>;

export function flattenGroupedData<T>(
    data: Record<string, T[]>,
): { label: string; data: T[] }[] {
    return Object.entries(data).map(([key, value]) => ({
        label: key,
        data: value,
    }));
}

export function toSlug(s: string) {
    return s.toLowerCase().replaceAll(" ", "-");
}

export function fromSlug(s: string) {
    s = s.replaceAll("-", " ");
    return s.charAt(0).toUpperCase() + s.slice(1);
}

export function deepMerge<T>(target: T, source: Partial<T>): T {
    for (const key in source) {
        if (
            source[key] &&
            typeof source[key] === "object" &&
            !Array.isArray(source[key])
        ) {
            target[key] = deepMerge(
                (target[key] as any) || {},
                source[key] as any,
            ) as any;
        } else {
            (target as any)[key] = source[key];
        }
    }
    return target;
}
