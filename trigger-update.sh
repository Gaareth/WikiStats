#!/bin/bash
set -e;


echo "> Pulling latest changes from git";
git pull origin main;


echo "> Building new Docker image"
docker compose --profile prod --build

echo "> Starting a test container"
docker run --rm --name wiki-stats-webserver-test -p 4322:4321 wiki-stats-webserver

echo "> Testing new container health..."
status_code=$(curl -o /dev/null -s -w "%{http_code}" "localhost:4322")

docker stop wiki-stats-webserver-test 2>/dev/null || true
docker rm wiki-stats-webserver-test 2>/dev/null || true

if [ "$status_code" -ge 200 ] && [ "$status_code" -lt 400 ]; then
    echo "URL returned 200 OK or any 300"
    echo "> Deploying new image to production"
    docker compose --profile prod up -d --build 
else
    echo -e "\033[31mURL did not return 200 OK or any 300. Status code: $status_code\033[0m"
    exit 1
fi
