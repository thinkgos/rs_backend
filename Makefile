postgress-skip:
	SKIP_DOCKER=true ./scripts/init_db.sh

.PHONY: 
	postgress-skip