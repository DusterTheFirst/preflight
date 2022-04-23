# preflight
An attempt at end to end testing of flight software with as little
setup as needed. It aims to be able to run SITL (Software In The Loop) simulations
interfacing with firmware directly with the ease of running unit tests. In the project's
current state it would not make sense to, but the system may grow to be able
to run HITL (Hardware In The Loop) simulations. This is built around Rust's powerful
macro system.

**This is an in development prototype, so expect it to be buggy.**

Documentation will be produced once there is a stable API/ABI

## Example
An example flight system can be found in the [`example/`](example/) directory

#### License
<sup>
    Licensed under the <a href="LICENSE">Mozilla Public License 2.0</a>
</sup>
