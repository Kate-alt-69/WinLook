CC ?= cc
AR ?= ar
CFLAGS ?= -std=c89 -Wall -Wextra -Werror -pedantic -O2
CPPFLAGS ?= -Iinclude -I../wl_Lanimate/include -I../wl_Lcpuv/include -I../wl_Lgpuv/include
BUILD_DIR ?= build
DEPENDENCY_SOURCES = $(wildcard ../wl_Lanimate/src/*.c) $(wildcard ../wl_Lcpuv/src/*.c) $(wildcard ../wl_Lgpuv/src/*.c)

all: $(BUILD_DIR)/libwl_Lwlde.a

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/wl_Lwlde.o: src/wl_Lwlde.c include/wl_Lwlde.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/libwl_Lwlde.a: $(BUILD_DIR)/wl_Lwlde.o
	$(AR) rcs $@ $^

$(BUILD_DIR)/test_wl_Lwlde: tests/test_wl_Lwlde.c src/wl_Lwlde.c $(DEPENDENCY_SOURCES) | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) $^ -lm -o $@

test: $(BUILD_DIR)/test_wl_Lwlde
	./$(BUILD_DIR)/test_wl_Lwlde

clean:
	rm -rf $(BUILD_DIR)

.PHONY: all test clean
