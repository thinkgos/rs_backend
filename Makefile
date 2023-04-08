postgress-skip:
	SKIP_DOCKER=true ./scripts/init_db.sh

sqlx-prepare:
# sqlx 离线元数据, 保存在 sqlx-data.json
	cargo sqlx prepare -- --lib 

.PHONY: 
	postgress-skip

