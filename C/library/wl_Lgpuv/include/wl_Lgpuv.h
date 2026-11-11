#ifndef WL_LGPUV_H
#define WL_LGPUV_H

/* C89-compatible GPU memory/submission admission core; caller serializes access. */
#define WL_LGPUV_VERSION_MAJOR 0
#define WL_LGPUV_VERSION_MINOR 1
#define WL_LGPUV_VERSION_PATCH 0

typedef enum wl_Lgpuv_work_class {
    WL_LGPUV_CRITICAL = 0,
    WL_LGPUV_INTERACTIVE = 1,
    WL_LGPUV_FOREGROUND = 2,
    WL_LGPUV_BACKGROUND = 3,
    WL_LGPUV_IDLE = 4
} wl_Lgpuv_work_class;

typedef enum wl_Lgpuv_result {
    WL_LGPUV_OK = 0,
    WL_LGPUV_INVALID_ARGUMENT = 1,
    WL_LGPUV_INVALID_POLICY = 2,
    WL_LGPUV_INVALID_ESTIMATE = 3,
    WL_LGPUV_BUDGET_EXCEEDED = 4,
    WL_LGPUV_QUEUE_FULL = 5,
    WL_LGPUV_UNKNOWN_TICKET = 6,
    WL_LGPUV_TICKET_EXHAUSTED = 7
} wl_Lgpuv_result;

typedef enum wl_Lgpuv_pressure {
    WL_LGPUV_PRESSURE_NORMAL = 0,
    WL_LGPUV_PRESSURE_CONSTRAINED = 1,
    WL_LGPUV_PRESSURE_CRITICAL = 2
} wl_Lgpuv_pressure;

typedef struct wl_Lgpuv_resource_slot {
    unsigned long id;
    unsigned long bytes;
    int active;
} wl_Lgpuv_resource_slot;

typedef struct wl_Lgpuv_submission_slot {
    unsigned long id;
    int active;
} wl_Lgpuv_submission_slot;

typedef struct wl_Lgpuv_governor {
    unsigned long memory_budget_bytes;
    unsigned long critical_reserve_bytes;
    unsigned long allocated_bytes;
    unsigned long next_ticket;
    unsigned int max_in_flight_submissions;
    unsigned int active_submissions;
    wl_Lgpuv_resource_slot *resources;
    unsigned int resource_capacity;
    wl_Lgpuv_submission_slot *submissions;
    unsigned int submission_capacity;
} wl_Lgpuv_governor;

wl_Lgpuv_result wl_Lgpuv_init(wl_Lgpuv_governor *governor,
    wl_Lgpuv_resource_slot *resources, unsigned int resource_capacity,
    wl_Lgpuv_submission_slot *submissions, unsigned int submission_capacity,
    unsigned long memory_budget_bytes, unsigned long critical_reserve_bytes,
    unsigned int max_in_flight_submissions);
wl_Lgpuv_result wl_Lgpuv_reserve(wl_Lgpuv_governor *governor,
    wl_Lgpuv_work_class work_class, unsigned long estimated_bytes, unsigned long *ticket);
wl_Lgpuv_result wl_Lgpuv_release(wl_Lgpuv_governor *governor, unsigned long ticket);
wl_Lgpuv_result wl_Lgpuv_begin_submission(wl_Lgpuv_governor *governor, unsigned long *ticket);
wl_Lgpuv_result wl_Lgpuv_complete_submission(wl_Lgpuv_governor *governor, unsigned long ticket);
wl_Lgpuv_result wl_Lgpuv_update_budget(wl_Lgpuv_governor *governor,
    unsigned long memory_budget_bytes, unsigned long critical_reserve_bytes);
unsigned long wl_Lgpuv_available_for(const wl_Lgpuv_governor *governor,
    wl_Lgpuv_work_class work_class);
wl_Lgpuv_pressure wl_Lgpuv_get_pressure(const wl_Lgpuv_governor *governor);

#endif
