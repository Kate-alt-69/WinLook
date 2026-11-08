CC ?= cc
CFLAGS ?= -std=c89 -Wall -Wextra -Werror -pedantic -O2
SOURCES = src/wl_Lanimate_math.c src/wl_Lanimate_tween.c src/wl_Lanimate_track.c

.PHONY: test clean

test: test_wl_Lanimate
	./test_wl_Lanimate

test_wl_Lanimate: $(SOURCES) tests/test_wl_Lanimate.c include/wl_Lanimate.h
	$(CC) $(CFLAGS) -Iinclude $(SOURCES) tests/test_wl_Lanimate.c -lm -o $@

clean:
	rm -f test_wl_Lanimate
