.PHONY: common client server

all: common client server

common:
	cd common && cargo build

client:
	cd client && cargo build

server:
	cd server && cargo build

