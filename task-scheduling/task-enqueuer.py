import subprocess
import dotenv
import re
import os

from tasks import (
    WIKI_CLI_BINARY,
    STATS_OUTPUT_PATH,
    SUPPORTED_WIKIS,
    process_wiki,
    SIMULATE,
    env_path,
)


def check_for_tasks():
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

        for dump_date in dumpdates_todo:
            if dump_date not in available_wikis_per_dump_date:
                available_wikis_per_dump_date[dump_date] = []
            available_wikis_per_dump_date[dump_date].append(wiki)

    if not available_wikis_per_dump_date:
        print("No tasks to enqueue.")
        return

    latest_dump_date = max(available_wikis_per_dump_date.keys())

    for dump_date, available_wikis in available_wikis_per_dump_date.items():

        # expected_wikis = wikis if latest_dump_date else only those that are available
        expected_wikis = (
            desired_wikis if dump_date == latest_dump_date else available_wikis
        )

        print(
            f"> Enqueuing: [{dump_date}]: {' '.join(available_wikis)}. Requires: {' '.join(expected_wikis)}"
        )

        for wiki in available_wikis:
            process_wiki.delay(wiki, dump_date, expected_wikis)
            # ...


def simulate_check_for_tasks():
    wikis = SUPPORTED_WIKIS.split(", ")
    for wiki in wikis:
        for dump_date in [
            "20250520",
            "20250601",
            "20250620",
            "20250701",
            "20250720",
            "20250801",
            "20250820",
        ]:
            print(f"> Enqueuing: [{wiki}] {dump_date}")
            process_wiki.delay(wiki, dump_date, wikis)


if __name__ == "__main__":
    # dump_date = "w"
    # dotenv.set_key(env_path, "IS_UPDATING", "false")
    # dotenv.set_key(env_path, "LATEST_DUMP", dump_date)

    # updated_wikis_dir = re.sub(r'(\d{8})', dump_date, os.getenv("DB_WIKIS_DIR"))
    # dotenv.set_key(env_path, "DB_WIKIS_DIR", updated_wikis_dir)

    if SIMULATE:
        simulate_check_for_tasks()
        # process_wiki.delay("jawiki", "20250901", ["dewiki", "jawiki"])
    else:
        check_for_tasks()
