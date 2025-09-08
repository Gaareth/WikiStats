#!/bin/bash

IMAGE_NAME="wiki-stats-webserver"
TEST_URL="localhost:4321"


print_error() {
    echo -e "\033[31m$1\033[0m"
}

cleanup() {
    echo "> Cleanup: stopping and removing container/image"
    docker stop $IMAGE_NAME 2>/dev/null || true
    docker rm $IMAGE_NAME 2>/dev/null || true
}

# Build the Docker image
docker build . -t $IMAGE_NAME

if [ $? -ne 0 ]; then
    print_error "Docker build failed. Stopping script."
    exit 1
fi


# Stop and remove any existing container named 'wiki-stats-webserver'
docker stop $IMAGE_NAME 2>/dev/null || true
docker rm $IMAGE_NAME 2>/dev/null || true

echo "> Starting container"
# Run the Docker container in detached mode
docker compose --profile prod up -d

# Wait a moment for the server to start
sleep 3

echo "> Checking return code"
# Check if the URL returns a 200 status
status_code=$(curl -o /dev/null -s -w "%{http_code}" "$TEST_URL")

if [ "$status_code" -ge 200 ] && [ "$status_code" -lt 400 ]; then
    echo "URL returned 200 OK or any 300"
else
    print_error "URL did not return 200 OK or any 300. Status code: $status_code"

    cleanup
    exit 1
fi

cleanup