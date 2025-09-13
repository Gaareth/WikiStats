
import json
from typing import Literal, Optional, TypedDict
from tasks import WIKI_TASKS_PREFIX, redis, REDIS_PREFIX, logger
import os
import subprocess
import time
import fcntl
from contextlib import contextmanager
import dotenv
from dotenv import load_dotenv
from env_vars import REBUILD_SERVER_BIN


class TaskData(TypedDict):
    status: Literal["RUNNING", "DONE", "FAILED", "QUEUED"]
    startedAt: Optional[str]
    dumpDate: str
    name: str
    finishedAt: Optional[str]
    message: Optional[str]

def set_task_status(data: TaskData):
    key = data["name"] + "_" + data["dumpDate"]
    redis.hset(WIKI_TASKS_PREFIX, mapping={key: json.dumps(data)})

def all_tasks_done():
    return all(
        value == "DONE" for value in redis.hvals(REDIS_PREFIX + "wiki-tasks")
    )


def check_all_sqlite_files_are_ready(supported_wikis, DB_DIR, name, dump_date):
    wikis_done = []
    for file in os.listdir(DB_DIR):
        filename = os.fsdecode(file)
        if filename.endswith(".sqlite") and os.path.getsize(
            os.path.join(DB_DIR, filename)
        ):
            wikis_done.append(filename.split("_")[0])

    logger.info(f"[{name} {dump_date}] wikis: done {wikis_done}")
    return set(supported_wikis).issubset(wikis_done)



def build_server():
    logger.info("> Rebuilding server")
    subprocess.run([REBUILD_SERVER_BIN], stdout=subprocess.DEVNULL)
    logger.info("Finished rebuilding server")



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