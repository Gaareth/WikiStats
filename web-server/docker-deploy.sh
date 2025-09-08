set -e
source ./.env

sh docker-build.sh && ssh ${DEPLOY_HOST} "cd ${DEPLOY_LOCATION}; ./after-deploy.sh"