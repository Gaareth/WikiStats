#!/bin/bash


echo "> Pulling latest changes from git";
#git pull origin main;

print_error() {
    echo -e "\033[31m$1\033[0m"
}

echo "> Building new Docker image"
docker compose --profile prod build

if [ $? -ne 0 ]; then
    print_error "Docker build failed. Stopping script."
    exit 1
fi

docker stop wiki-stats-webserver-test 2>/dev/null || true
docker rm wiki-stats-webserver-test 2>/dev/null || true

echo "> Starting a test container"
docker compose run -d --name wiki-stats-webserver-test -p 4322:4321 webserver 
TEST_URL="localhost:4322"

# wait a moment for the server to start
sleep 3

echo "> Testing new container health..."
status_code=$(curl -o /dev/null -s -w "%{http_code}" "$TEST_URL")
echo "Status code: $status_code"

docker stop wiki-stats-webserver-test 2>/dev/null || true
docker rm wiki-stats-webserver-test 2>/dev/null || true


if [ "$status_code" -ge 200 ] && [ "$status_code" -lt 400 ]; then
    echo "URL returned 200 OK or any 300"
    echo "> Deploying new image to production"
    docker compose --profile prod up -d --build 
else
    print_error "URL did not return 200 OK or any 300. Status code: $status_code"
    exit 1
fi
