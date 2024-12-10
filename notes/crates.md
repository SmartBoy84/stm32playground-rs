Check out https://rtic.rs/dev/book/en/ as well
https://blog.theembeddedrustacean.com/stm32f4-embedded-rust-at-the-hal-the-rtic-framework?source=more_series_bottom_blogs

Read [this](https://mcyoung.xyz/2021/06/01/linker-script/)

[Comparison](https://willhart.io/post/embedded-rust-options/)

# Cortex-M-rt
    use cortex-m crate => "provides start code and minimal runtime for C-M based MCUs"
    TODO: experiment with other places to store (checkout memory map and "advanced usage")

# Cortex-M
Provides low level access to Cortex-M processor
    Shouldn't really have to use this
Most importantly, provides a critical section implementation - this is why it's used
Critical section contains code that is run by multiple threads
    Contains shared variables (think global variables)

# rtt_target
IO from MCU -> debug probe -> computer'


# Panic-probe
`panic-probe = {version = "0.3.2", features = ["print-defmt"]}`  
Embedded panic handler  
Automatically exits probe-run on panic   

## Features  
- `print-defmt`: pretty clear; instruct it to output messages using defmt

# defmt-rtt
src: https://defmt.ferrous-systems.com/setup.html#global_logger  
defmt crate requires application link to a `global_logger`  
Most debug probes (including st-link) support rtt-protocol   
=> Use defmt-rtt -> provides a `global_logger` compatible with defmt

# Emabssy-stm32
`embassy-stm32 = {version = "0.1.0", features = ["defmt", "stm32f302cbtx", "memory-x", "time-driver-any"]}`

## Features
- Memory-x => deals with cration of the `memory.x` linker script (required by cortex-m)
- time-driver-any => embassy uses a hardware timer to expose to the embassy-time crate
    => as per datasheet, all timers not equal - if wanting to use another [check](https://docs.embassy.dev/embassy-stm32/git/stm32c011d6/index.html#time)
    => [!NOTE] currently let embassy pick timer - should I be explicit?  
- exti => external interrupts
- defmt => use defmt for logging

# Embassy-executor
`embassy-executor = {version = "0.6.3", features = ["arch-cortex-m", "executor-thread", "integrated-timers"]}`

From: [embassy_executor](https://docs.rs/embassy-executor/latest/embassy_executor/#architecture) docs  
"Async/await executor designed for embedded use"  

## Features  
- `integrated-timers` => implements traits given [here](https://docs.embassy.dev/embassy-time-queue-driver/git/default/index.html)
    - it's a timer queue implementation
    - apparently there's also `generic-queue` in `embassy-time` => *TODO* - figure out differences?

## 1. Select an [arch](https://docs.rs/embassy-executor/latest/embassy_executor/#architecture)
Will use `arch-cortex-m` feature  
Also `arch-spin` which is platform agnostic -> never sleeps on idle  

## 2. Select an executor
- Interrupt-mode executor => idk, figure out but seems complicated
- Thread-mode executor (`executor-thread`) => ez pz
    - Uses `WFE` (wait for event) inst to sleep when no more work to do  
    - To wake up when task received, `sev` (send event) inst executed to wake thread and execute task  

# Embassy time
`embassy-time = {version = "0.3.2", features = ["tick-hz-32_768"]}`  

## Features
- `tick-hz-32_768"` - [Apparently](https://crates.io/crates/embassy-stm32) default tick rate of `embassy-time` (1MHz) could cause problems with timers - 32kHz good enough  

# embassy-embedded-hal  
In my fight to get embedded_sdmmc working, I eventually found out that this crate was needed  
It provides utilities which enable interop with embedded_hal traits => so crates based on it.  

# embassy_sync
Rn, just need it to create a device in embassy_embedded_hal (prevent multiple different writes to same bus)  
It's pretty cool learning this - you jump from error->error learning about the ecosystem  

## Mutex
[Multiple types](https://docs.embassy.dev/embassy-sync/git/default/mutex/struct.Mutex.html)  
Right now, I'm using `NoopRawMutex` as apparently that's safe when Mutex is held in single executor  
Come back to this when using multiple executors (what does that mean) -> different priority tasks  

Also I use `blocking_mutex` as so I can use embedded_hal crates (only blocking SpiDevice impl embedded_hal::SpiDevice<Word>)
=> will this be a problem? What's the difference

# embedded_sdmmc  
`embedded-sdmmc = {version = "0.8.1", default-features = false, features = ["defmt-log"]}`  
##  Features  
- `defmt-log`: required for printing various info (e.g., CardType)
- `default-features = false`: `log` feature enabled by default but incompatible with `defmt-log`