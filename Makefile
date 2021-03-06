image:
	docker build . --tag judge:1.0.0

clean:
	docker rm -f $$(docker ps -aq)

build:
	mkdir -p /home/rinne/Sphinx/code
	mkdir -p /home/rinne/Sphinx/core

	gcc Core.c -o /home/rinne/Sphinx/core/core -lpthread -O2 -Wall
	gcc Core2.c -o /home/rinne/Sphinx/core/core2 -lpthread -O2 -Wall
	g++ Jury.cpp -o /home/rinne/Sphinx/core/Jury -O2 -Wall -std=c++17

RunTest:
	cargo test --release -- --nocapture

RunZoo:
	cd ~/kafka_2.12-2.3.0 && \
	bin/zookeeper-server-start.sh config/zookeeper.properties

RunKafka:
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-server-start.sh config/server.properties
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-topics.sh --create --zookeeper localhost:2181 --replication-factor 1 --partitions 1 --topic in
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-topics.sh --create --zookeeper localhost:2181 --replication-factor 1 --partitions 1 --topic result

StopZoo:
	cd ~/kafka_2.12-2.3.0 && \
	bin/zookeeper-server-stop.sh

list:
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-topics.sh --list --zookeeper localhost:2181
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-topics.sh --describe --zookeeper localhost:2181 --topic in
	cd ~/kafka_2.12-2.3.0 && \
	bin/kafka-topics.sh --describe --zookeeper localhost:2181 --topic result

kafka_config_compose_file:
	python3 ./deployments/sphinx-core-boj/scripts/kafka_cluster.py generate --src deployments/sphinx-core-boj/kafka-docker-deployment.template.yaml --dst deployments/sphinx-core-boj/kafka-docker-deployment.yaml

kafka_up:
	docker-compose -f deployments/sphinx-core-boj/kafka-docker-deployment.yaml up

kafka_start:
	docker-compose -f deployments/sphinx-core-boj/kafka-docker-deployment.yaml start

kafka_stop:
	docker-compose -f deployments/sphinx-core-boj/kafka-docker-deployment.yaml stop

kafka_down:
	docker-compose -f deployments/sphinx-core-boj/kafka-docker-deployment.yaml down

kafka_clean: kafka_down
	sudo rm -r /home/rinne/data/kafka-data/*


.PHONY: kafka_config_compose_file kafka_up kafka_down kafka_start kafka_stop kafka_clean
.PHONY: image
.PHONY: clean
.PHONY: build
.PHONY: RunTest
.PHONY: RunZoo
.PHONY: RunKafka
.PHONY: StopZoo
.PHONY: list