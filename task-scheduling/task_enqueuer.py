from datetime import datetime, timezone
import subprocess
import argparse


from tasks import (
    WIKI_CLI_BINARY,
    STATS_OUTPUT_PATH,
    SUPPORTED_WIKIS,
    WIKI_TASKS_PREFIX,
    redis,
    set_task_status
)


def check_for_tasks():
    from tasks import process_wiki  # TODO: put in shared
    desired_wikis = SUPPORTED_WIKIS.split(", ")
    available_wikis_per_dump_date = {}

    for wiki in desired_wikis:
        args = [
            WIKI_CLI_BINARY,
            "get-tasks",
            "--stats-path",
            STATS_OUTPUT_PATH,
            "-w",
            wiki,
        ]
        print(f"Checking for tasks: {' '.join(args)}")
        result = subprocess.run(
            args,
            capture_output=True,
            text=True,
        )
        print(f"Output for {wiki}: {result.stdout}")
        dumpdates_todo = [
            d.strip('"').strip()
            for d in result.stdout.split("Todo dumpdates: [")[1]
            .rstrip("]\n")
            .split(", ")
        ]
        dumpdates_todo = [d for d in dumpdates_todo if len(d) == 8]
        

        for dump_date in dumpdates_todo:
            if dump_date not in available_wikis_per_dump_date:
                available_wikis_per_dump_date[dump_date] = []
            available_wikis_per_dump_date[dump_date].append(wiki)

    now_utc_seconds = int(datetime.now(timezone.utc).timestamp())
    redis.set(f"{WIKI_TASKS_PREFIX}:last_checked_for_tasks", now_utc_seconds)

    if not available_wikis_per_dump_date:
        print("No tasks to enqueue.")
        return

    latest_dump_date = max(available_wikis_per_dump_date.keys())

    for dump_date, available_wikis in available_wikis_per_dump_date.items():
        if len(dump_date) != 8:
            print(f"Skipping invalid dump date: {dump_date}")
            continue

        # expected_wikis = wikis if latest_dump_date else only those that are available
        expected_wikis = (
            desired_wikis if dump_date == latest_dump_date else available_wikis
        )

        print(
            f"> Enqueuing: [{dump_date}]: {' '.join(available_wikis)}. Requires: {' '.join(expected_wikis)}"
        )

        for wiki in available_wikis:
            process_wiki.delay(wiki, dump_date, expected_wikis)
            set_task_status({
                "name": wiki,
                "dumpDate": dump_date,
                "status": "QUEUED",
                "startedAt": None,
                "finishedAt": None,
                "message": None,
            })
            

def simulate_check_for_tasks():
    from tasks import process_wiki  # TODO: put in shared
    wikis = SUPPORTED_WIKIS.split(", ")
    for wiki in wikis:
        for dump_date in [
            "20250520",
            "20250701",
            "20250720",
            "20250801",
            "20250820",
        ]:
            print(f"> Enqueuing: [{wiki}] {dump_date}")
            process_wiki.delay(wiki, dump_date, wikis)
            set_task_status({
                "name": wiki,
                "dumpDate": dump_date,
                "status": "QUEUED",
                "startedAt": None,
                "finishedAt": None,
                "message": None,
            })


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Task Enqueuer")
    parser.add_argument("--dummy", action="store_true", help="Enqueue dummy tasks instead of real ones")
    args = parser.parse_args()
    dummy = args.dummy

    if dummy:
        print("Enqueuing dummy tasks")
        simulate_check_for_tasks()
        # process_wiki.delay("jawiki", "20250901", ["dewiki", "jawiki"])
    else:
        check_for_tasks()
