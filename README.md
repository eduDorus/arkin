# Arkin

# Docker ClickHouse
docker run -d -v "$(pwd)/clickhouse/logs:/var/log/clickhouse-server" -v "$(pwd)/clickhouse/data:/var/lib/clickhouse" -e CLICKHOUSE_DB=arkin -e CLICKHOUSE_USER=arkin_admin -e CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1 -e CLICKHOUSE_PASSWORD=test1234 -p 18123:8123 -p 19000:9000 --user ${UID}:${GID} --name arkin-clickhouse-server --ulimit nofile=262144:262144 clickhouse/clickhouse-server:latest

docker exec -it arkin-clickhouse-server clickhouse-client