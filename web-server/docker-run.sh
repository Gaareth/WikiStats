./docker-build;
docker run -p 3000:4321 -e DB_PATH=file:/database/prod.db -v book-store:/database wikistats-astro
