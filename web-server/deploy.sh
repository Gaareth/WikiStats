set -e
source ../.env

# npm run build && rsync -v -r package.json package-lock.json node_modules dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run build && rsync -v -r package.json package-lock.json dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run astro check;
#npm run build;
#./validate-docker-setup.sh


# upgrade patch?version
# jq '.version |= (. | split(".") | .[2] = ((.[2] | tonumber) + 1 | tostring) | join("."))' package.json | sponge package.json

git push   # push first

echo "> Select version bump type:"
select bump in patch minor major; do
    if [[ -n "$bump" ]]; then
        npm version "$bump"   # creates new commit + tag
        git push && git push --tags  # push commit and tag
        break
    else
        echo "Invalid selection."
    fi
done

# rsync --exclude node_modules --exclude dist --delete -v -r ./* ${DEPLOY_HOST}:${DEPLOY_LOCATION}/web-server-test-environment;
ssh ${DEPLOY_HOST} "cd ${DEPLOY_LOCATION}; ./trigger-update.sh"