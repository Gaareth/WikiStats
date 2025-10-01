# WikiStats
Wikipedia statistics and shortest path finder


## Installation
1. Clone the repository
` git clone https://github.com/Gaareth/WikiStats.git --recurse-submodules`
2. Copy `env.example` to `.env` and edit the variables as needed
3. Run `docker compose --profile prod up -d --build` to build and start the services
4. (Optional) Create systemd service for task-scheduling, e.g., for the celery worker and for celery beat

## Update
- Run `./trigger-update.sh`


## Architecture

- web-server: Astro webserver
- task-scheduling: Celery, Redis, RabbitMQ scheduler wikipedia dump processing
- wiki-stats-rs: Rust services for processing wikipedia dumps and a blazingly fast webserver calculating shortest paths 
- igraph: Graph experiements