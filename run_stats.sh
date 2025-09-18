set -e
source ../.env

read -p "Enter dumpdate: " dumpdate
./binaries/cli stats -o ${STATS_OUTPUT_PATH}/${dumpdate}.json -d ${DB_WIKIS_DIR} --all-wikis
