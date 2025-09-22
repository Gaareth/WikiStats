redis-cli keys "CELERY-WIKI_*" | xargs redis-cli del

if [ -z "$VIRTUAL_ENV" ]; then
    if [ -d ".venv" ]; then
        source .venv/bin/activate
    else
        python3 -m venv .venv
        source .venv/bin/activate
        pip install -r requirements.txt
    fi
fi

celery -A tasks purge -f
echo "Cleared all CELERY-WIKI_* keys from Redis and purged all Celery tasks."