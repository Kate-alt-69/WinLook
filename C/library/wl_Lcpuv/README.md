# wl_Lcpuv

C89-compatible, caller-storage CPU admission/accounting core. The host supplies a fixed ticket-slot array and serializes access; the library allocates no memory, creates no threads and does not control OS scheduling or unrelated processes.

Costs are abstract caller-defined work units. Completion reports measured cost and whether it exceeded the estimate. Do not pass wall-clock duration as CPU time unless the host has explicitly defined that as its policy unit.

Build/test with `make test` or CMake/CTest. Windows-specific worker-pool and CPU-time adapters are separate future work.
