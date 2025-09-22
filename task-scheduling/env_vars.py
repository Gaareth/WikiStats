import os
from dotenv import load_dotenv
from pathlib import Path

env_path = Path("../.env")
load_dotenv(dotenv_path=env_path)



WIKI_CLI_BINARY = os.getenv("WIKI_CLI_BINARY")
if WIKI_CLI_BINARY is None:
    print("WIKI_CLI_BINARY env var can't be None")
    exit(-1)

WIKI_BASEPATH = os.getenv("DB_WIKIS_BASEPATH")
if WIKI_BASEPATH is None:
    print("DB_WIKIS_BASEPATH env var can't be None")
    exit(-1)

STATS_OUTPUT_PATH = os.getenv("STATS_OUTPUT_PATH")
if STATS_OUTPUT_PATH is None:
    print("STATS_OUTPUT_PATH env var can't be None")
    exit(-1)

SUPPORTED_WIKIS = os.getenv("SUPPORTED_WIKIS")
if SUPPORTED_WIKIS is None:
    print("SUPPORTED_WIKIS env var can't be None")
    exit(-1)

REBUILD_SERVER_BIN = os.getenv("REBUILD_SERVER_BIN")
if REBUILD_SERVER_BIN is None:
    print("REBUILD_SERVER_BIN env var can't be None")
    exit(-1)

DB_WIKIS_DIR = os.getenv("DB_WIKIS_DIR")
if DB_WIKIS_DIR is None:
    print("DB_WIKIS_DIR env var can't be None")
    exit(-1)

SIMULATE = os.getenv("TASK_SCHEDULING_SIMULATE") == "true"
if os.getenv("TASK_SCHEDULING_SIMULATE") is None:
    print("TASK_SCHEDULING_SIMULATE env var can't be None")
    exit(-1)


CRONTAB_SCHEDULE_STR = os.getenv("TASK_SCHEDULING_SCHEDULE")
if CRONTAB_SCHEDULE_STR is None:
    print("TASK_SCHEDULING_SCHEDULE env var can't be None")
    exit(-1)

UPDATE_DONE_SH = "../update-done.sh"

TABLES = ["pagelinks", "page", "linktarget"]