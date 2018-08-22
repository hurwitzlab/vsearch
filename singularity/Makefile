SINGULARITY = /usr/local/bin/singularity
IMG = vsearch-2.8.2.img
DEF = image.def
SIZE = 1024

run:
	sudo $(SINGULARITY) run $(IMG)

clean:
	rm -f $(IMG)

img: clean
	#sudo $(SINGULARITY) build $(IMG) $(DEF)
	sudo $(SINGULARITY) create --size $(SIZE) $(IMG)
	sudo $(SINGULARITY) bootstrap $(IMG) $(DEF)

shell:
	sudo $(SINGULARITY) shell --writable -B $(shell pwd):/home $(IMG)
