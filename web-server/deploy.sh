set -e
source ../.env

# npm run build && rsync -v -r package.json package-lock.json node_modules dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run build && rsync -v -r package.json package-lock.json dist ${DEPLOY_HOST}:${DEPLOY_LOCATION}

# npm run astro check;
#npm run build;
#./validate-docker-setup.sh


# upgrade patch?version
# jq '.version |= (. | split(".") | .[2] = ((.[2] | tonumber) + 1 | tostring) | join("."))' package.json | sponge package.json

echo "> Select version bump type:"
select bump in patch minor major; do
    if [[ -n "$bump" ]]; then
        echo "Selected bump type: $bump"/
        break
    else
        echo "Invalid selection."
    fi
done

git push

# if push worked bump version
npm version "$bump";
git add package.json package-lock.json; git commit -m "Bump version to $(jq -r .version package.json)";
git push --no-verify && git push --tags  # push commit and tag

# rsync --exclude node_modules --exclude dist --delete -v -r ./* ${DEPLOY_HOST}:${DEPLOY_LOCATION}/web-server-test-environment;
ssh ${DEPLOY_HOST} "cd ${DEPLOY_LOCATION}; ./trigger-update.sh"