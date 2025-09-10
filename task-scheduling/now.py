import argparse
import time

from tasks import process_wiki, finish_dump

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Process a list of strings.")

    # Add the list of strings argument
    parser.add_argument('wikis', nargs='*',
                        help='The wikis to update')
    parser.add_argument('--dump-date',
                        help='Dump date for stats generation')

    args = parser.parse_args()
    wikis = args.wikis

    print("List of wikis:", wikis)

    for i, task_name in enumerate(wikis):
        process_wiki.delay(task_name, args.dump_date, wikis)
        print(f"Scheduled {task_name} now!")
        time.sleep(1)

    if len(wikis) == 0:
        print(args.dump_date)
        finish_dump.delay(args.dump_date)
