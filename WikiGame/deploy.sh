#cross build  --bin server --target aarch64-unknown-linux-gnu --release
#cargo build --bin server --release
cross build  --bin server --target x86_64-unknown-linux-gnu --release
#rsync -vvv target/aarch64-unknown-linux-gnu/release/server gareth@192.168.178.40:/home/gareth/wiki-stats
#rsync -vvv target/release/server gareth@192.168.178.94:/home/gareth/wiki-stats
rsync -vvv target/x86_64-unknown-linux-gnu/release/server gareth@192.168.178.94:/home/gareth/wiki-stats
