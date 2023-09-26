clean:
	rm -rf target
	rm -rf media

RO=target/release/

$(RO)test:
	cargo build --release

media/: media/graph.svg

media/graph.svg: $(RO)test
	mkdir -p $(@D)
	$(RO)test | neato -Tsvg > $@
