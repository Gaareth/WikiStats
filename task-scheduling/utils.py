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
from env_vars import REBUILD_SERVER_BIN, STATS_OUTPUT_PATH, WIKI_BASEPATH
import re
from typing import Callable


class TaskData(TypedDict):
    status: Literal["RUNNING", "DONE", "FAILED", "QUEUED"]
    startedAt: Optional[str]
    dumpDate: Optional[str]
    name: str
    finishedAt: Optional[str]
    message: Optional[str]


def set_task_status(data: TaskData):
    dump_date = data["dumpDate"] if data["dumpDate"] is not None else ""
    key = data["name"] + "_" + dump_date
    redis.hset(WIKI_TASKS_PREFIX, mapping={key: json.dumps(data)})


def all_tasks_done():
    return all(value == "DONE" for value in redis.hvals(REDIS_PREFIX + "wiki-tasks"))


def get_dump_dates_without_web_wiki_sizes():
    def does_not_contain_web_wiki_sizes(filepath):
        try:
            with open(filepath, "r") as f:
                data = json.load(f)
            return "web_wiki_sizes" not in data or data["web_wiki_sizes"] is None
        except Exception:
            return False

    return get_done_dump_dates(does_not_contain_web_wiki_sizes)

def get_dump_dates_without_samples_stats():
    def does_not_contain_samples_stats(filepath):
        try:
            with open(filepath, "r") as f:
                data = json.load(f)
            return "bfs_sample_stats" not in data or data["bfs_sample_stats"] is None
        except Exception:
            return False
        
    def has_sqlite_files(filepath):
        _, filename = os.path.split(filepath)
        dumpdate = filename.split(".json")[0]
        db_dir = os.path.join(WIKI_BASEPATH, dumpdate, "sqlite")
        return os.path.isdir(db_dir) and len(os.listdir(db_dir)) > 0

    return get_done_dump_dates(lambda fp: does_not_contain_samples_stats(fp) and has_sqlite_files(fp))


def get_done_dump_dates(filter: (Callable[[str], bool]) = lambda _: True):
    wikis_done_total: list[str] = []
    stats_file_pattern = r"(\d{8}).json"
    
    for file in os.listdir(STATS_OUTPUT_PATH):
        filename = os.fsdecode(file)
        file_path = os.path.join(STATS_OUTPUT_PATH, filename)
        if re.match(stats_file_pattern, filename) and filter(file_path):
            wikis_done_total.append(filename.split(".json")[0])
    return wikis_done_total

def get_done_wikis(dump_date):
    wikis_done_total = []
    with open(os.path.join(STATS_OUTPUT_PATH, f"{dump_date}.json"), "r") as f:
         data = json.load(f)
         wikis_done_total = data["wikis"]

    return wikis_done_total

def check_stats_are_ready(supported_wikis, DB_DIR, name, dump_date):
    already_done = get_done_wikis(dump_date)
    sqlite_done = get_sqlite_files(DB_DIR)
    wikis_done = already_done + sqlite_done
    logger.info(f"[{name} {dump_date}] wikis: done {wikis_done}")
    return set(supported_wikis).issubset(wikis_done)


def get_sqlite_files(DB_DIR):
    wikis_done = []
    for file in os.listdir(DB_DIR):
        filename = os.fsdecode(file)
        if filename.endswith(".sqlite") and os.path.getsize(
            os.path.join(DB_DIR, filename)
        ):
            wikis_done.append(filename.split("_")[0])

    return wikis_done

def get_latest_dump_date(DB_WIKIS_BASEPATH):
    dump_dates: list[int] = []
    dump_date_pattern = r"(\d{8})"
    for file in os.listdir(DB_WIKIS_BASEPATH):
        filename = os.fsdecode(file)
        if os.path.isdir(os.path.join(DB_WIKIS_BASEPATH, filename)) and re.match(
            dump_date_pattern, filename
        ):
            dump_dates.append(int(filename))

    return max(dump_dates) if dump_dates else None


def deleted_old_dump_dates(keep_date, DB_WIKIS_BASEPATH, STATS_OUTPUT_PATH):
    """Delete dump date directories older than keep_date if their stats file exists"""
    # Warning: If the new dump covers less wikis than before, this will result in less wikis in the end
    # However, I will ignore this for now
    dump_date_pattern = r"(\d{8})"
    for file in os.listdir(DB_WIKIS_BASEPATH):
        filename = os.fsdecode(file)
        filepath = os.path.join(DB_WIKIS_BASEPATH, filename)
        if os.path.isdir(filepath) and re.match(
            dump_date_pattern, filename
        ):
            stats_exists = os.path.exists(
                os.path.join(STATS_OUTPUT_PATH, f"{filename}.json")
            )
            if not stats_exists:
                logger.warning(
                    f"Stats file for dump date {filename} does not exist, skipping deletion."
                )
                continue
            
            date_int = int(filename)
            if date_int < keep_date:
                logger.info(f"Deleting old dump date directory: {filepath}")
                subprocess.run(["rm", "-rf", filepath])


def build_server():
    logger.info("> Rebuilding server")    
    redis.set(f"{REDIS_PREFIX}:is-rebuilding", "true")
    subprocess.run([REBUILD_SERVER_BIN], stdout=subprocess.DEVNULL)
    logger.info("Finished rebuilding server")
    redis.set(f"{REDIS_PREFIX}:is-rebuilding", "false")


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
