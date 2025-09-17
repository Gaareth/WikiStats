import { createSignal, type Accessor, type JSX } from "solid-js";
import { cn } from "../utils";

type Column<T> = {
    key: keyof T;
    label: string;
    isSortable?: boolean;
    render?: (item: any) => JSX.Element | string;
};

type SortConfig<T> = {
    key: keyof T | null;
    direction: "asc" | "desc";
};

type TableProps<T> = {
    data: T[];
    columns: Column<T>[];
    sortConfig?: SortConfig<T>;
};

export function Table<T extends Record<string, any>>(props: TableProps<T>) {
    const [sortConfig, setSortConfig] = createSignal<SortConfig<T>>(
        props.sortConfig || {
            key: null,
            direction: "asc",
        },
    );

    const sortedData: Accessor<T[]> = () => {
        const { key, direction } = sortConfig();
        if (!key) return props.data;

        return [...props.data].sort((a, b) => {
            if (a[key] < b[key]) return direction === "asc" ? -1 : 1;
            if (a[key] > b[key]) return direction === "asc" ? 1 : -1;
            return 0;
        });
    };

    const handleSort = (key: keyof T) => {
        setSortConfig((prev) => {
            if (prev.key === key) {
                return {
                    key,
                    direction: prev.direction === "asc" ? "desc" : "asc",
                };
            }
            return { key, direction: "asc" };
        });
    };

    const padding = "px-1.5 py-1.5 sm:px-4 sm:py-2";

    return (
        <div class="overflow-x-auto text-sm sm:text-lg">
            <table class="min-w-full border border-gray-300 rounded-lg">
                <thead class="bg-gray-100 dark-layer-2">
                    <tr>
                        {props.columns.map((col) => (
                            <th
                                class={cn(
                                    "border text-left group dark-border-2",
                                    padding,
                                    col.isSortable && "cursor-pointer",
                                )}
                                onClick={() =>
                                    col.isSortable && handleSort(col.key)
                                }>
                                {col.label}{" "}
                                <span
                                    class={cn(
                                        "hidden",
                                        col.isSortable &&
                                            "group-hover:inline-block group-hover:opacity-50",
                                        sortConfig().key === col.key &&
                                            col.isSortable &&
                                            "inline-block",
                                    )}>
                                    {sortConfig().direction === "asc"
                                        ? "↑"
                                        : "↓"}
                                </span>
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {sortedData().map((row) => (
                        <tr>
                            {props.columns.map((col) => (
                                <td class={cn("border dark-border-1", padding)}>
                                    {col.render
                                        ? col.render(row[col.key])
                                        : row[col.key]}
                                </td>
                            ))}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
