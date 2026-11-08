CC ?= cc
AR ?= ar
CFLAGS ?= -std=c89 -Wall -Wextra -Werror -pedantic -O2
CPPFLAGS ?= -Iinclude
BUILD_DIR ?= build
SOURCES = src/wl_Lanimate_math.c src/wl_Lanimate_tween.c src/wl_Lanimate_track.c
OBJECTS = $(patsubst src/%.c,$(BUILD_DIR)/%.o,$(SOURCES))
LIBRARY = $(BUILD_DIR)/libwl_Lanimate.a
TEST = $(BUILD_DIR)/test_wl_Lanimate

.PHONY: all static test clean

all: static

static: $(LIBRARY)

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/%.o: src/%.c include/wl_Lanimate.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(LIBRARY): $(OBJECTS)
	$(AR) rcs $@ $^

$(TEST): tests/test_wl_Lanimate.c $(LIBRARY) include/wl_Lanimate.h
	$(CC) $(CPPFLAGS) $(CFLAGS) tests/test_wl_Lanimate.c $(LIBRARY) -lm -o $@

test: $(TEST)
	./$(TEST)

clean:
	rm -rf $(BUILD_DIR)
