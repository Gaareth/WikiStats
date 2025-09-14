#!/bin/bash
source .env

# Ensure logs directory exists with user permissions
mkdir -p ./logs
chown -R $(whoami):$(whoami) ./logs

cp .env ./web-server/.env

print_error() {
    echo -e "\033[31m$1\033[0m"
}

echo "> Pulling latest changes from git";
git pull origin main;
git submodule update --init --recursive

if [ $? -ne 0 ]; then
    print_error "Git pull failed. Stopping script."
    exit 1
fi


echo "> Building new Docker image"
docker compose --profile prod build

if [ $? -ne 0 ]; then
    print_error "Docker build failed. Stopping script."
    exit 1
fi

# if no docker container with the name wiki-stats-webserver is running, start it and quit
if ! [ "$(docker ps -q -f name=wiki-stats-webserver)" ]; then
    echo "> Starting container for the first time"
    docker compose --profile prod up -d
    echo "Successfully started all containers."
    exit 0
fi

echo "Info: There is already a container running with the name wiki-stats-webserver. Proceeding to test and deploy new version."

docker stop wiki-stats-webserver-test 2>/dev/null || true
docker rm wiki-stats-webserver-test 2>/dev/null || true


echo "> Starting a test container"
docker compose run -d --name wiki-stats-webserver-test -p 4322:4321 webserver 
TEST_URL="localhost:4322"

echo "> Testing new container health..."

# Wait for the server to be ready with health check
echo "> Waiting for server to be ready..."
max_attempts=30
attempt=0
while [ $attempt -lt $max_attempts ]; do
    if curl -f -s "$TEST_URL" > /dev/null 2>&1; then
        echo "Server is ready after $((attempt + 1)) attempts"
        break
    fi
    attempt=$((attempt + 1))
    echo "Attempt $attempt/$max_attempts - Server not ready yet, waiting 1 second..."
    sleep 1
done

if [ $attempt -eq $max_attempts ]; then
    print_error "Server failed to start within 30 seconds"
    docker stop wiki-stats-webserver-test 2>/dev/null || true
    docker rm wiki-stats-webserver-test 2>/dev/null || true
    exit 1
fi


status_code=$(curl -o /dev/null -s -w "%{http_code}" "$TEST_URL")
echo "Status code: $status_code"

docker stop wiki-stats-webserver-test 2>/dev/null || true
docker rm wiki-stats-webserver-test 2>/dev/null || true


if [ "$status_code" -ge 200 ] && [ "$status_code" -lt 400 ]; then
    echo "URL returned 200 OK or any 300"
    echo "> Deploying new image to production"
    docker compose --profile prod up -d
    echo "Successfully deployed new version."
else
    print_error "URL did not return 200 OK or any 300. Status code: $status_code"
    exit 1
fi
