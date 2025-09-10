#!/bin/bash

# Activate venv if not already active
if [ -z "$VIRTUAL_ENV" ]; then
    if [ -d "venv" ]; then
        source venv/bin/activate
    else
        python3 -m venv venv
        source venv/bin/activate
    fi
fi

celery -A tasks worker --loglevel=info --concurrency 1