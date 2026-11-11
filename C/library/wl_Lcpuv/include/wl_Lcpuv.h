#ifndef WL_LCPUV_H
#define WL_LCPUV_H

/* C89-compatible, allocation-free admission core. Caller serializes access. */
#define WL_LCPUV_VERSION_MAJOR 0
#define WL_LCPUV_VERSION_MINOR 1
#define WL_LCPUV_VERSION_PATCH 0

typedef enum wl_Lcpuv_work_class {
    WL_LCPUV_CRITICAL = 0,
    WL_LCPUV_INTERACTIVE = 1,
    WL_LCPUV_FOREGROUND = 2,
    WL_LCPUV_BACKGROUND = 3,
    WL_LCPUV_IDLE = 4
} wl_Lcpuv_work_class;

typedef enum wl_Lcpuv_result {
    WL_LCPUV_OK = 0,
    WL_LCPUV_INVALID_ARGUMENT = 1,
    WL_LCPUV_INVALID_POLICY = 2,
    WL_LCPUV_INVALID_ESTIMATE = 3,
    WL_LCPUV_BUDGET_EXCEEDED = 4,
    WL_LCPUV_CONCURRENCY_LIMIT = 5,
    WL_LCPUV_UNKNOWN_TICKET = 6,
    WL_LCPUV_TICKET_EXHAUSTED = 7
} wl_Lcpuv_result;

typedef enum wl_Lcpuv_pressure {
    WL_LCPUV_PRESSURE_NORMAL = 0,
    WL_LCPUV_PRESSURE_CONSTRAINED = 1,
    WL_LCPUV_PRESSURE_CRITICAL = 2
} wl_Lcpuv_pressure;

typedef struct wl_Lcpuv_slot {
    unsigned long id;
    unsigned long estimate;
    int active;
} wl_Lcpuv_slot;

typedef struct wl_Lcpuv_governor {
    unsigned long window_budget;
    unsigned long critical_reserve;
    unsigned long spent;
    unsigned long reserved;
    unsigned long next_ticket;
    unsigned int max_in_flight;
    unsigned int active_count;
    wl_Lcpuv_slot *slots;
    unsigned int slot_capacity;
} wl_Lcpuv_governor;

/* Caller supplies slots storage; no heap allocation or hidden threads. */
wl_Lcpuv_result wl_Lcpuv_init(wl_Lcpuv_governor *governor,
    wl_Lcpuv_slot *slots, unsigned int slot_capacity,
    unsigned long window_budget, unsigned long critical_reserve,
    unsigned int max_in_flight);
wl_Lcpuv_result wl_Lcpuv_try_acquire(wl_Lcpuv_governor *governor,
    wl_Lcpuv_work_class work_class, unsigned long estimate, unsigned long *ticket);
wl_Lcpuv_result wl_Lcpuv_complete(wl_Lcpuv_governor *governor,
    unsigned long ticket, unsigned long actual, int *overrun);
wl_Lcpuv_result wl_Lcpuv_cancel(wl_Lcpuv_governor *governor, unsigned long ticket);
void wl_Lcpuv_reset_window(wl_Lcpuv_governor *governor);
unsigned long wl_Lcpuv_available_for(const wl_Lcpuv_governor *governor,
    wl_Lcpuv_work_class work_class);
wl_Lcpuv_pressure wl_Lcpuv_get_pressure(const wl_Lcpuv_governor *governor);

#endif
