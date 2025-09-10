set -e
source ../.env

# npm run build && rsync -v -r package.json package-lock.json node_modules dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run build && rsync -v -r package.json package-lock.json dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run astro check;
#npm run build;
#./validate-docker-setup.sh


# upgrade patch?version
# jq '.version |= (. | split(".") | .[2] = ((.[2] | tonumber) + 1 | tostring) | join("."))' package.json | sponge package.json
npm version patch;
git push;

# rsync --exclude node_modules --exclude dist --delete -v -r ./* ${DEPLOY_HOST}:${DEPLOY_LOCATION}/web-server-test-environment;
ssh ${DEPLOY_HOST} "cd ${DEPLOY_LOCATION}; ./trigger-update.sh"