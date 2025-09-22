import redis, { type RedisClientType } from "redis";

export let redisClient: RedisClientType;
export const REDIS_PREFIX = "WIKI:";

(async () => {
    const redis_host = process.env.REDIS_HOST || "localhost";
    const redis_port = +(process.env.REDIS_PORT || 6379);

    console.log(
        "Connecting to redis at host:",
        redis_host,
        "port:",
        redis_port,
    );
    redisClient = redis.createClient({
        socket: {
            host: redis_host,
            port: redis_port,
        },
    });

    redisClient.on("error", (error) =>
        console.error(
            `Error connecting to redis at ${redis_host}:${redis_port}: ${error}`,
        ),
    );

    await redisClient.connect();
})();
