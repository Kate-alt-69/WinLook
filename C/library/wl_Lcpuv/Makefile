CC ?= cc
AR ?= ar
CFLAGS ?= -std=c89 -Wall -Wextra -Werror -pedantic -O2
CPPFLAGS ?= -Iinclude
BUILD_DIR ?= build
all: $(BUILD_DIR)/libwl_Lcpuv.a
$(BUILD_DIR):
	mkdir -p $@
$(BUILD_DIR)/wl_Lcpuv.o: src/wl_Lcpuv.c include/wl_Lcpuv.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@
$(BUILD_DIR)/libwl_Lcpuv.a: $(BUILD_DIR)/wl_Lcpuv.o
	$(AR) rcs $@ $^
$(BUILD_DIR)/test_wl_Lcpuv: tests/test_wl_Lcpuv.c $(BUILD_DIR)/libwl_Lcpuv.a
	$(CC) $(CPPFLAGS) $(CFLAGS) $< $(BUILD_DIR)/libwl_Lcpuv.a -o $@
test: $(BUILD_DIR)/test_wl_Lcpuv
	./$(BUILD_DIR)/test_wl_Lcpuv
clean:
	rm -rf $(BUILD_DIR)
.PHONY: all test clean
