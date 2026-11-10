CC ?= cc
AR ?= ar
CFLAGS ?= -std=c89 -Wall -Wextra -Werror -pedantic -O2
CPPFLAGS ?= -Iinclude
BUILD_DIR ?= build
all: $(BUILD_DIR)/libwl_Lgpuv.a
$(BUILD_DIR):
	mkdir -p $@
$(BUILD_DIR)/wl_Lgpuv.o: src/wl_Lgpuv.c include/wl_Lgpuv.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@
$(BUILD_DIR)/libwl_Lgpuv.a: $(BUILD_DIR)/wl_Lgpuv.o
	$(AR) rcs $@ $^
$(BUILD_DIR)/test_wl_Lgpuv: tests/test_wl_Lgpuv.c $(BUILD_DIR)/libwl_Lgpuv.a
	$(CC) $(CPPFLAGS) $(CFLAGS) $< $(BUILD_DIR)/libwl_Lgpuv.a -o $@
test: $(BUILD_DIR)/test_wl_Lgpuv
	./$(BUILD_DIR)/test_wl_Lgpuv
clean:
	rm -rf $(BUILD_DIR)
.PHONY: all test clean
