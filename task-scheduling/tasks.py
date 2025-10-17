import json
import logging
import os
import re
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Literal, Optional, TypedDict
from contextlib import contextmanager

import redis
from celery import Celery
from celery.utils.log import get_task_logger
from celery.schedules import crontab

from env_vars import (
    ADD_SAMPLE_STATS_END_HOUR,
    ADD_SAMPLE_STATS_START_HOUR,
    WIKI_CLI_BINARY,
    WIKI_BASEPATH,
    STATS_OUTPUT_PATH,
    SUPPORTED_WIKIS,
    REBUILD_SERVER_BIN,
    DB_WIKIS_DIR,
    SIMULATE,
    UPDATE_DONE_SH,
    TABLES,
    env_path,
    CRONTAB_SCHEDULE_STR,
)


logger = get_task_logger(__name__)


REDIS_PREFIX = "CELERY-WIKI_"
WIKI_TASKS_PREFIX = REDIS_PREFIX + "wiki-tasks"

broker_host = os.getenv("RABBITMQ_HOST", "localhost")
broker_port = os.getenv("RABBITMQ_PORT", "5672")

redis_host = os.getenv("REDIS_HOST", "localhost")
redis_port = os.getenv("REDIS_PORT", "6379")
redis_db = os.getenv("REDIS_DB", "0")

app = Celery(
    "tasks",
    broker=f"amqp://guest:guest@{broker_host}:{broker_port}//",
    backend=f"redis://{redis_host}:{redis_port}/{redis_db}",
)

app.conf.result_backend_transport_options = {"global_keyprefix": REDIS_PREFIX}
app.conf.update(
    broker_url=f"amqp://guest:guest@{broker_host}:{broker_port}//",
    broker_transport_options={
        # "visibility_timeout": 7200,  # 2 hours (use if long tasks)
        "heartbeat": 30,  # keep connection alive
        "consumer_timeout": 31536000000000,
    },
    task_acks_late=True,  # ACK only after task finishes
    task_reject_on_worker_lost=True,  # requeue if worker dies
)


from celery.signals import after_setup_logger, after_setup_task_logger

LOG_FORMAT = "%(levelname)s [%(name)s] %(message)s"

@after_setup_logger.connect
def setup_root_logger(logger, *args, **kwargs):
    for handler in logger.handlers:
        handler.setFormatter(logging.Formatter(LOG_FORMAT))

@after_setup_task_logger.connect
def setup_task_logger(logger, *args, **kwargs):
    for handler in logger.handlers:
        handler.setFormatter(logging.Formatter(LOG_FORMAT))


redis = redis.Redis(host=redis_host, port=redis_port, db=redis_db)


import handlers  # imports the signal handlers
from utils import (
    TaskData,
    check_stats_are_ready,
    deleted_old_dump_dates,
    get_done_dump_dates,
    get_dump_dates_without_samples_stats,
    get_dump_dates_without_web_wiki_sizes,
    set_task_status,
    all_tasks_done,
    build_server,
    file_lock,
    safe_set_env_vars,
    get_latest_dump_date,
    should_generate_stats,
)
from task_enqueuer import check_for_tasks, check_latest_dump_date_is_fully_complete


@app.on_after_configure.connect
def setup_periodic_tasks(sender: Celery, **kwargs):
    print(f"Setting up periodic tasks with schedule {CRONTAB_SCHEDULE_STR}")
    schedule = CRONTAB_SCHEDULE_STR.split()
    if len(schedule) != 5:
        print(
            f"TASK_SCHEDULING_SCHEDULE is invalid, must have 5 parts but has {len(schedule)}: {CRONTAB_SCHEDULE_STR}"
        )
        exit(-1)
    minute, hour, day_of_month, month, day_of_week = schedule
    sender.add_periodic_task(
        crontab(
            minute=minute,
            hour=hour,
            day_of_month=day_of_month,
            month_of_year=month,
            day_of_week=day_of_week,
        ),
        enqueuing_task.s(),
    )


@app.task
def add_sample_stats(dump_date: str):
    data: TaskData = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": "ADD SAMPLE STATS",
        "finishedAt": None,
        "message": None,
    }
    set_task_status(data)

    logger.info(f"Running add_sample_stats for dump date {dump_date}")
    output_file = os.path.join(STATS_OUTPUT_PATH, f"{dump_date}.json")
    db_path = os.path.join(WIKI_BASEPATH, dump_date, "sqlite")

    sample_size = "200"
    num_threads = "10"

    cmd = [
        WIKI_CLI_BINARY,
        "stats",
        "add-sample-stats",
        "-o",
        output_file,
        "--db-path",
        db_path,
        "--sample-size",
        sample_size,
        "--threads",
        num_threads,
        "--all-wikis"
    ]
    logger.info(f"> Running command: {cmd}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    logger.info(f"Command output: {result.stdout}")
    if result.returncode != 0:
        data["status"] = "FAILED"
        data["message"] = result.stderr
        set_task_status(data)
        raise Exception(result.stderr)

    data["status"] = "DONE"
    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    set_task_status(data)
    build_server()


def is_within_sample_stats_window():
    if ADD_SAMPLE_STATS_START_HOUR is None or ADD_SAMPLE_STATS_END_HOUR is None:
        return False

    now = datetime.now()
    start = now.replace(hour=int(ADD_SAMPLE_STATS_START_HOUR), minute=0, second=0, microsecond=0)
    end = now.replace(hour=int(ADD_SAMPLE_STATS_END_HOUR), minute=0, second=0, microsecond=0)
    return start <= now < end

@app.task
def add_web_wiki_sizes(dump_date: str):
    data: TaskData = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": "ADD WEB WIKI SIZES",
        "finishedAt": None,
        "message": None,
    }
    set_task_status(data)

    logger.info(f"Running add_web_wiki_sizes for dump date {dump_date}")
    output_file = os.path.join(STATS_OUTPUT_PATH, f"{dump_date}.json")
    sqlite_and_downloads_parent_path = os.path.join(WIKI_BASEPATH, dump_date)

    cmd = [
        WIKI_CLI_BINARY,
        "stats",
        "add-web-wiki-sizes",
        "-o",
        output_file,
        "--base-path",
        sqlite_and_downloads_parent_path,
        "--dump-date",
        dump_date,
    ]
    logger.info(f"> Running command: {cmd}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    logger.info(f"Command output: {result.stdout}")
    if result.returncode != 0:
        data["status"] = "FAILED"
        data["message"] = result.stderr
        set_task_status(data)
        raise Exception(result.stderr)

    data["status"] = "DONE"
    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    set_task_status(data)
    build_server()


def enqueue_web_sizes():
    # only add wiki sizes for the latest dump date that doesn't have them, as it takes about 8mins and does a lot of requests
    # this can be changed in future
    # also the sqlite and download dir will likely be deleted 
    todo_dump_dates = get_dump_dates_without_web_wiki_sizes()
    done_dump_dates = get_done_dump_dates()

    if len(done_dump_dates) > 0:
        latest_done_dump_date = sorted(done_dump_dates)[-1]
        if latest_done_dump_date in todo_dump_dates and check_latest_dump_date_is_fully_complete():
            add_web_wiki_sizes.delay(latest_done_dump_date)
            data: TaskData = {
                "status": "QUEUED",
                "startedAt": datetime.now(timezone.utc).isoformat(),
                "dumpDate": latest_done_dump_date,
                "name": "ADD WEB WIKI SIZES",
                "finishedAt": None,
                "message": None,
            }
            set_task_status(data)

def enqueue_sample_stats():
    if is_within_sample_stats_window():
        todo_dump_dates = get_dump_dates_without_samples_stats()
        for todo_dump_date in todo_dump_dates:
            add_sample_stats.delay(todo_dump_date)
            data: TaskData = {
                "status": "QUEUED",
                "startedAt": datetime.now(timezone.utc).isoformat(),
                "dumpDate": todo_dump_date,
                "name": "ADD SAMPLE STATS",
                "finishedAt": None,
                "message": None,
            }
            set_task_status(data)

@app.task
def enqueuing_task():
    data: TaskData = {
        "name": "TASK CHECKER",
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": None,
        "finishedAt": None,
        "message": None,
    }
    set_task_status(data)

    try:
        check_for_tasks()
        enqueue_web_sizes()
        enqueue_sample_stats()
    except Exception as e:
        data["status"] = "FAILED"
        data["message"] = f"Task failed"
        set_task_status(data)
        raise e

    data["status"] = "DONE"
    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    set_task_status(data)


@app.task
def finish_dump(dump_date):
    # set redis key for this dumpdate so only one task does this
    if redis.get(f"{WIKI_TASKS_PREFIX}:{dump_date}") == b"RUNNING":
        logger.info(f"STATISTIC GENERATION Task for {dump_date} is already running")
        return

    redis.set(f"{WIKI_TASKS_PREFIX}:{dump_date}", "RUNNING")

    logger.info("Running statistic generation task")
    name = "STATISTIC GENERATION"
    data: TaskData = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": name,
        "finishedAt": None,
        "message": None,
    }
    set_task_status(data)


    try:
        if not SIMULATE:
            output_file = os.path.join(STATS_OUTPUT_PATH, f"{dump_date}.json")
            db_path = os.path.join(WIKI_BASEPATH, dump_date, "sqlite")
            cmd = [
                WIKI_CLI_BINARY,
                "stats",
                "generate",
                "-o",
                output_file,
                "--db-path",
                db_path,
                "--all-wikis",
            ]
            logger.info(f"> Running command: {cmd}")
            result = subprocess.run(cmd, capture_output=True, text=True)
            logger.info(f"Command output: {result.stdout}")
            if result.returncode != 0:
                raise Exception(result.stderr)
        else:
            time.sleep(10)
        data["status"] = "DONE"

    except Exception as e:
        logger.error(f"Exception: {e}")
        data["message"] = f"Task failed. {e}"
        data["status"] = "FAILED"
        set_task_status(data)
        redis.set(f"{WIKI_TASKS_PREFIX}:{dump_date}", "FAILED")
        return -1

    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    set_task_status(data)

    # new_name = STATS_OUTPUT_PATH.split(".json")[0] + "-" + str(datetime.now(timezone.utc).timestamp()) + ".json"
    # subprocess.run(["cp", STATS_OUTPUT_PATH, new_name])

    # result = subprocess.run(["redis-cli", "KEYS", f"{REDIS_PREFIX}:*", "|", "xargs", "redis-cli",  "DEL"])
    # result = subprocess.run([UPDATE_DONE_SH])

    redis.set(f"{WIKI_TASKS_PREFIX}:{dump_date}", "DONE")
    # redis.hset(REDIS_PREFIX + "wiki-tasks-status", mapping={dump_date: "DONE"})

    # all_done = all_tasks_done()

    # check if dump_date is the latest date and then delete all others older than it if their stats file exists
    latest_dump_date = get_latest_dump_date(WIKI_BASEPATH)
    logger.info(
        f"[!!!!] INFO: latest dump date is {latest_dump_date}. Current dump date is {dump_date}."
    )
    if latest_dump_date is None or int(dump_date) >= latest_dump_date:
        logger.info(
            f"[!!!!] INFO: {dump_date} is the latest dump date, cleaning up old dump dates"
        )
        updated_wikis_dir = re.sub(r"(\d{8})", dump_date, DB_WIKIS_DIR)
        deleted_old_dump_dates(int(dump_date), WIKI_BASEPATH, STATS_OUTPUT_PATH)

        safe_set_env_vars(
            env_path,
            {
                "DB_WIKIS_DIR": updated_wikis_dir,
            },
        )

    time.sleep(2)
    if not SIMULATE:
        build_server()

    # return result.returncode
    return 200


@app.task(bind=True)
def process_wiki(self, name, dump_date, supported_wikis: List[str]):
    logger.info(f"Received tasks for {name} {dump_date}")
    # different from DB_WIKIS_DIR as that is for the current dumpdate
    DB_DIR = os.path.join(WIKI_BASEPATH, dump_date, "sqlite")
    os.makedirs(DB_DIR, exist_ok=True)

    # status per dumpdate, as finish_dump also works per dumpdate
    # redis.hset(REDIS_PREFIX + "wiki-tasks-status", mapping={dump_date: "RUNNING"})

    data: TaskData = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": name,
        "finishedAt": None,
        "message": None,
    }
    set_task_status(data)

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
                "--remove-downloads",
                "--validate",
                "--num-pages",
                "2"
            ]
            logging.info(f"> Running command: {cmd}")
            result = subprocess.run(cmd, capture_output=True, text=True)
            # logging.info(result)
            if result.stdout:
                logger.info(f"Command output: {result.stdout}")
            if result.stderr:
                logger.error(f"Command error output: {result.stderr}")
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
        logger.error(f"Exception: {e}")
        data["status"] = "FAILED"
        data["message"] = f"Task failed. {e}"
        set_task_status(data)
        raise e

    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    set_task_status(data)

    now_utc_seconds = int(datetime.now(timezone.utc).timestamp())
    redis.hset(f"{WIKI_TASKS_PREFIX}:last-updated", mapping={name: now_utc_seconds})

    # if this tasks is the last of all SUPPORTED_WIKIS, then finish_dump
    # it is the last if all of supportedwikis are in wikis_done
    if should_generate_stats(supported_wikis, DB_DIR, name, dump_date):
        logging.info(f"[{name} {dump_date}] done")
        finish_dump(dump_date)

    return 200
