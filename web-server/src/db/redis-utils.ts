import type { TaskType } from "../components/SPC/DumpProgress/Task/Task.astro";
import { redisClient } from "./redis";

export async function getTasks() {
    const keys = await redisClient.hGetAll("CELERY-WIKI_wiki-tasks");
    const tasks: TaskType[] = Object.entries(keys).map(([key, value]) => {
        let { name, status, startedAt, finishedAt, ...rest } =
            JSON.parse(value);
        return {
            name,
            status,
            startedAt: new Date(startedAt),
            finishedAt: finishedAt != null ? new Date(finishedAt) : undefined,
            ...rest,
        };
    });

    return tasks;
}

export async function getLastChecked() {
    let last_checked = await redisClient.get(
        "CELERY-WIKI_wiki-tasks:last_checked_for_tasks",
    );
    let last_checked_date = last_checked
        ? new Date(Number(last_checked) * 1000)
        : null;
    return last_checked_date;
}
