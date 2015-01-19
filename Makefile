default: all

all: deps build

deps:
    (cd lib/rustful && make deps && make)

build:
    rustc -L lib/rustful/lib/ -o profile profile.rs

.PHONY: all deps build
