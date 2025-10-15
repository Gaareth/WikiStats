# WikiStats [![wakatime](https://wakatime.com/badge/user/58cbdf08-bbb1-4720-96b5-3e6c96e7f148/project/fe252dde-c610-43d7-ab13-cda697080d5a.svg)](https://wakatime.com/badge/user/58cbdf08-bbb1-4720-96b5-3e6c96e7f148/project/fe252dde-c610-43d7-ab13-cda697080d5a)
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

## Todo
- Properly handle wikiprefixes: currently alot of stuff depends on wikiprefixes being two chars, and ending with "wiki". Also their url might be different
- More tests, especially benchmarking
- Download command
- Code refactor

## Disclaimer

The dumps may not always fully reflect the current links online, especially for websites that are frequently updated. 
### Example: 
https://it.wikipedia.org/wiki/Nati_il_21_aprile
Misses e.g. a link to https://it.wikipedia.org/wiki/Wesllem, as the it was removed shortly before the dump was made. And then later readded. 

Demonstration:
```
 grep -R --color=always -o -E '.{0,20}Wesllem.{0,8}' itwiki-20251001-linktarget.sql
_John'),(1020575,0,'Wesllem'),(1094
```

=> linktarget id of Wesllem is 1020575.

Does pagelinks.sql contain any links from Nati_il_21_aprile (pageid=1276495) to this linktarget_id?

```

gareth@manjaro-kde ~/dev/WikiStats/wiki-folder/20251001/downloads main*
❯ grep -o -E '\(1276495,[0-9]+,1020575\)' itwiki-20251001-pagelinks.sql

gareth@manjaro-kde ~/dev/WikiStats/wiki-folder/20251001/downloads main* 6s
❯ 
```

Does not seem like it?

### Other Examples
Validating on ukwiki 20251001:
Всесвітня премія фентезі за найкращу повість -> Премія Урсули К. Ле Ґуїн missing from db

This link is also not in the dump, but Премія Урсули К. Ле Ґуїн is on the page in a navbox at the bottom. Which might be the reason its not in the dump?

Also the latest update was the 26th october 2024, and its not frequently updated.

The dump only contains with Ґуїн in the name:
Всесвітня премія фентезі за найкращу повість -> Урсула_Ле_Ґуїн

