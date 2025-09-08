#cargo build --bin cli --release
cross build  --bin cli --target x86_64-unknown-linux-gnu --release
#rsync -vvv target/release/cli gareth@192.168.178.94:/home/gareth/wiki-stats
rsync -vvv target/x86_64-unknown-linux-gnu/release/cli gareth@192.168.178.94:/home/gareth/wiki-stats

rsync --mkpath task-scheduling/requirements.txt task-scheduling/tasks.py task-scheduling/now.py task-scheduling/task-schedule.json  gareth@192.168.178.94:/home/gareth/wiki-stats/task-scheduling/