set -e;

npm run build && ./docker-build-local.sh;
npm version patch;
git push;
npm run build;
echo "> Uploading v$(jq -r .version package.json)"
docker buildx build  --platform=linux/amd64 . -t ghcr.io/gaareth/wiki-stats-webserver:$(jq -r .version package.json) -t ghcr.io/gaareth/wiki-stats-webserver:latest --push;
