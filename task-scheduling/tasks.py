import json
import logging
import os
import re
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import List
import fcntl
from contextlib import contextmanager

import dotenv
import redis
from celery import Celery
from dotenv import load_dotenv
from celery.utils.log import get_task_logger

logger = get_task_logger(__name__)

REDIS_PREFIX = "CELERY-WIKI_"

app = Celery("tasks", broker="amqp://localhost", backend="redis://localhost")
app.conf.result_backend_transport_options = {"global_keyprefix": REDIS_PREFIX}
app.conf.update(
    broker_url="amqp://guest:guest@localhost:5672//",
    broker_transport_options={
        "visibility_timeout": 7200,  # 2 hours (use if long tasks)
        "heartbeat": 30,  # keep connection alive
    },
    task_acks_late=True,  # ACK only after task finishes
    task_reject_on_worker_lost=True,  # requeue if worker dies
    task_time_limit=7200,  # optional: hard limit for tasks
    task_soft_time_limit=7100,  # optional: soft limit before kill
)


redis = redis.Redis(host="localhost", port=6379, db=0)

TASK_SCHEDULE_PATH = "./task-schedule.json"
with open(TASK_SCHEDULE_PATH) as f:
    task_schedule = json.load(f)


def all_tasks_done():
    return all(
        value != "RUNNING" for value in redis.hvals(REDIS_PREFIX + "wiki-tasks-status")
    )


# @app.on_after_configure.connect
# def setup_periodic_tasks(sender, **kwargs):
#     for i, task in enumerate(task_schedule):
#         crontab_string = task["crontab"].split(" ")
#         sender.add_periodic_task(
#             crontab(minute=crontab_string[0], hour=crontab_string[1],
#                     day_of_week=crontab_string[2], day_of_month=crontab_string[3],
#                     month_of_year=crontab_string[4]),
#             process_wiki.s(task["name"], i, len(task_schedule)),
#         )
#         print(f"Scheduled {task['name']} for {task['crontab']}")
#     # sender.add_periodic_task(crontab(minute="*"), process_wiki.s('dewiki', 1))
#     # sender.add_periodic_task(10.0, process_wiki.s('enwiki', 2))


env_path = Path("../.env")
# env_path = Path("/home/gareth/dev/webdev/astro/wiki/.env")
load_dotenv(dotenv_path=env_path)

# WIKI_CLI_BINARY = "../cli"
WIKI_CLI_BINARY = "../target/release/cli"

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

if os.getenv("DB_WIKIS_DIR") is None:
    print("DB_WIKIS_DIR env var can't be None")
    exit(-1)

SIMULATE = False

UPDATE_DONE_SH = "../update-done.sh"


@contextmanager
def file_lock(file_path):
    """Context manager for file locking to prevent concurrent access to .env file"""
    lock_path = str(file_path) + ".lock"
    with open(lock_path, "w") as lock_file:
        try:
            # Acquire exclusive lock
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)
            yield
        finally:
            # Lock is automatically released when file is closed
            pass


def safe_set_env_vars(env_path, updates):
    """Safely update multiple environment variables with file locking"""
    with file_lock(env_path):
        # Load current env to ensure we don't lose other variables
        load_dotenv(dotenv_path=env_path, override=True)

        # Apply all updates
        for key, value in updates.items():
            dotenv.set_key(env_path, key, value)

        # Small delay to ensure file write is complete
        time.sleep(0.1)


@app.task
def finish_dump(dump_date):
    # set redis key for this dumpdate so only one task does this
    if redis.get(REDIS_PREFIX + f"wiki-tasks:{dump_date}") == b"RUNNING":
        logger.info(f"STATISTIC GENERATION Task for {dump_date} is already running")
        return

    redis.set(REDIS_PREFIX + f"wiki-tasks:{dump_date}", "RUNNING")

    logger.info("Running statistic generation task")
    name = "STATISTIC GENERATION"
    data = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": name,
    }
    key = name + "_" + dump_date
    redis.hset(REDIS_PREFIX + "wiki-tasks", mapping={key: json.dumps(data)})

    try:
        if not SIMULATE:
            output_file = os.path.join(STATS_OUTPUT_PATH, f"{dump_date}.json")
            db_path = os.path.join(WIKI_BASEPATH, dump_date, "sqlite")
            cmd = [
                WIKI_CLI_BINARY,
                "stats",
                "-o",
                output_file,
                "-d",
                db_path,
                "--all-wikis",
            ]
            logger.info(f"> Running command: {cmd}")
            result = subprocess.run(cmd, capture_output=True, text=True)
            # logger.info(f"Command output: {result.stdout}")
            if result.returncode != 0:
                logger.error(f"Error: Command output: {result.stderr}")

        else:
            time.sleep(10)
        data["status"] = "DONE"

    except Exception as e:
        data["status"] = "FAILED"
        return -1

    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    redis.hset(REDIS_PREFIX + "wiki-tasks", mapping={key: json.dumps(data)})

    # new_name = STATS_OUTPUT_PATH.split(".json")[0] + "-" + str(datetime.now(timezone.utc).timestamp()) + ".json"
    # subprocess.run(["cp", STATS_OUTPUT_PATH, new_name])

    # result = subprocess.run(["redis-cli", "KEYS", f"{REDIS_PREFIX}:*", "|", "xargs", "redis-cli",  "DEL"])
    # result = subprocess.run([UPDATE_DONE_SH])

    redis.set(REDIS_PREFIX + f"wiki-tasks:{dump_date}", "DONE")
    redis.hset(REDIS_PREFIX + "wiki-tasks-status", mapping={dump_date: "DONE"})

    all_done = all_tasks_done()
    updated_wikis_dir = re.sub(r"(\d{8})", dump_date, os.getenv("DB_WIKIS_DIR") or "")

    env_updates = {
        "IS_UPDATING": "false" if all_done else "true",
        "LATEST_DUMP": dump_date,
        "DB_WIKIS_DIR": updated_wikis_dir,
    }

    safe_set_env_vars(env_path, env_updates)

    time.sleep(2)
    # build_server()

    # return result.returncode
    return 200


def build_server():
    logger.info("> Rebuilding server")
    subprocess.run([REBUILD_SERVER_BIN], stdout=subprocess.DEVNULL)
    logger.info("Finished rebuilding server")


TABLES = ["pagelinks", "page", "linktarget"]

# todo schedule so its in the night (the retry)
# 2 hours
INCOMPLETE_DUMP_WAIT_S = 2 * 60 * 60



@app.task(bind=True)
def process_wiki(self, name, dump_date, supported_wikis: List[str]):
    logger.info(f"Received tasks for {name} {dump_date}")
    DB_DIR = os.path.join(WIKI_BASEPATH, dump_date, "sqlite")
    os.makedirs(DB_DIR, exist_ok=True)

    # logger.info(DB_DIR)

    is_updating = os.getenv("IS_UPDATING")
    if is_updating == "true" and not SIMULATE:
        try:
            safe_set_env_vars(env_path, {"IS_UPDATING": "true"})
            build_server()
        except Exception as e:
            logger.error(f"Failed building server {e} ")

    # status per dumpdate, as finish_dump also works per dumpdate
    redis.hset(REDIS_PREFIX + "wiki-tasks-status", mapping={dump_date: "RUNNING"})

    data = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": name,
    }

    key = name + "_" + dump_date
    redis.hset(REDIS_PREFIX + "wiki-tasks", mapping={key: json.dumps(data)})

    retry_exception = None
    try:
        if not SIMULATE:
            cmd = [
                WIKI_CLI_BINARY,
                "process-databases",
                "-w",
                name,
                "--path",
                WIKI_BASEPATH,
                "--dump-date",
                dump_date,
            ]
            logging.info(f"> Running command: {cmd}")
            result = subprocess.run(cmd, capture_output=True, text=True)
            logging.info(result)
            if result.returncode != 0:
                raise Exception(result.stderr)
        else:
            # logger.info(db_file)
            db_file = os.path.join(DB_DIR, f"{name}_TESTdatabase.sqlite")
            os.makedirs(os.path.dirname(db_file), exist_ok=True)
            if not os.path.exists(db_file) or (
                os.path.exists(db_file) and os.path.getsize(db_file) == 0
            ):
                with open(db_file, "w") as f:
                    f.write("foo")

            time.sleep(5)
            logger.info("simulate: done write to file")
        data["status"] = "DONE"

    except Exception as e:
        print(f"Exception: {e}")
        data["status"] = "FAILED"
        retry_exception = e

    data["finishedAt"] = datetime.now(timezone.utc).isoformat()

    redis.hset(REDIS_PREFIX + "wiki-tasks", mapping={key: json.dumps(data)})

    now_utc_seconds = int(datetime.now(timezone.utc).timestamp())
    redis.hset(REDIS_PREFIX + "wiki-tasks-last-updated", mapping={name: now_utc_seconds})

    if retry_exception is not None:
        ONE_DAY_S = 24 * 60 * 60
        data["message"] = f"Task failed. Retry in {ONE_DAY_S}s"
        redis.hset(REDIS_PREFIX + "wiki-tasks", mapping={key: json.dumps(data)})
        logging.error(f"Failed task  {name}, retry in one day. Error:", retry_exception)
        raise self.retry(exc=Exception(retry_exception), countdown=ONE_DAY_S)


    # print(f"Tasks done: {tasks_done}/{num_tasks}")
    # if tasks_done >= num_tasks:
    #     finish_dump(captured_dt)
    #     redis.set(REDIS_PREFIX + "tasks-done-count", 0)

    # if this tasks is the last of all SUPPORTED_WIKIS, then finish_dump
    # it is the last if all of supportedwikis are in wikis_done
    if check_all_are_ready(supported_wikis, DB_DIR, name, dump_date):
        logging.info(f"[{name} {dump_date}] done")
        finish_dump(dump_date)

    return 200


def check_all_are_ready(supported_wikis, DB_DIR, name, dump_date):
    wikis_done = []
    for file in os.listdir(DB_DIR):
        filename = os.fsdecode(file)
        if filename.endswith(".sqlite") and os.path.getsize(
            os.path.join(DB_DIR, filename)
        ):
            wikis_done.append(filename.split("_")[0])

    logger.info(f"[{name} {dump_date}] wikis: done {wikis_done}")
    return set(supported_wikis).issubset(wikis_done)
