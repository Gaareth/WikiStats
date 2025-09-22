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
    "tasks", broker=f"amqp://guest:guest@{broker_host}:{broker_port}//", backend=f"redis://{redis_host}:{redis_port}/{redis_db}"
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


redis = redis.Redis(host=redis_host, port=redis_port, db=redis_db)


import handlers  # imports the signal handlers
from utils import (
    TaskData,
    check_all_sqlite_files_are_ready,
    deleted_old_dump_dates,
    set_task_status,
    all_tasks_done,
    build_server,
    file_lock,
    safe_set_env_vars,
    get_latest_dump_date,
)
from task_enqueuer import check_for_tasks


@app.on_after_configure.connect
def setup_periodic_tasks(sender: Celery, **kwargs):
    logger.info(f"Setting up periodic tasks with schedule {CRONTAB_SCHEDULE_STR}")
    schedule = CRONTAB_SCHEDULE_STR.split()
    if len(schedule) != 5:
        logger.error(
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
        task_enqueuer.s(),
    )


@app.task
def task_enqueuer():
    check_for_tasks()


@app.task
def finish_dump(dump_date):
    # set redis key for this dumpdate so only one task does this
    if redis.get(f"{WIKI_TASKS_PREFIX}:{dump_date}") == b"RUNNING":
        logger.info(f"STATISTIC GENERATION Task for {dump_date} is already running")
        return

    redis.set(f"{WIKI_TASKS_PREFIX}:{dump_date}", "RUNNING")

    logger.info("Running statistic generation task")
    name = "STATISTIC GENERATION"
    data = {
        "status": "RUNNING",
        "startedAt": datetime.now(timezone.utc).isoformat(),
        "dumpDate": dump_date,
        "name": name,
    }
    key = name + "_" + dump_date
    redis.hset(WIKI_TASKS_PREFIX, mapping={key: json.dumps(data)})

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
            logger.info(f"Command output: {result.stdout}")
            if result.returncode != 0:
                raise Exception(result.stderr)
        else:
            time.sleep(10)
        data["status"] = "DONE"

    except Exception as e:
        data["status"] = "FAILED"
        return -1

    data["finishedAt"] = datetime.now(timezone.utc).isoformat()
    redis.hset(WIKI_TASKS_PREFIX, mapping={key: json.dumps(data)})

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

        safe_set_env_vars(env_path, {
            "DB_WIKIS_DIR": updated_wikis_dir,
        })

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
    if check_all_sqlite_files_are_ready(supported_wikis, DB_DIR, name, dump_date):
        logging.info(f"[{name} {dump_date}] done")
        finish_dump(dump_date)

    return 200
