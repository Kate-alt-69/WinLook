# wl_Lgpuv

C89-compatible, caller-storage GPU memory/submission admission core. The host provides resource and submission slot arrays and serializes access. The library does not allocate graphics resources, call DXGI, submit commands, create threads or guarantee residency.

A resource lease accounts for estimated bytes only. The renderer must release the actual graphics resource before releasing its lease. Submission tickets are completed only after the renderer's fence/completion signal. A later Windows adapter will feed OS-reported budgets and device-loss events.

Build/test with `make test` or CMake/CTest.
