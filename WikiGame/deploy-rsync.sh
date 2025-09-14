source ../.env

cross build  --bin cli --target ${DEPLOY_TARGET_ARCH} --release
cross build  --bin server --target ${DEPLOY_TARGET_ARCH} --release
rsync -vvv target/${DEPLOY_TARGET_ARCH}/release/server ${DEPLOY_HOST}:${DEPLOY_LOCATION}/binaries/server
rsync -vvv target/${DEPLOY_TARGET_ARCH}/release/cli ${DEPLOY_HOST}:${DEPLOY_LOCATION}/binaries/cli
