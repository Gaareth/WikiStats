import argparse
import time

from tasks import process_wiki, task_schedule, finish_dump

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Process a list of strings.")

    # Add the list of strings argument
    parser.add_argument('wikis', nargs='*',
                        help='The wikis to update')
    parser.add_argument('--dump-date',
                        help='Dump date for stats generation (only when len(wikis) == 0)')

    args = parser.parse_args()
    wikis = args.wikis
    if len(wikis) == 0 and args.dump_date is None:
        wikis = [t["name"] for t in task_schedule]

    print("List of wikis:", wikis)

    for i, task_name in enumerate(wikis):
        force = len(args.wikis) > 0
        if force:
            print("Forced!")

        process_wiki.delay(task_name, i, len(wikis), skip_ready_check=force)
        print(f"Scheduled {task_name} now!")
        time.sleep(1)

    if len(wikis) == 0:
        print(args.dump_date)
        finish_dump.delay(args.dump_date)
