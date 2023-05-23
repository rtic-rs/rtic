# RTIC vs. the world

RTIC aims to provide the lowest level of abstraction needed for developing robust and reliable embedded software.

It provides a minimal set of required mechanisms for safe sharing of mutable resources among interrupts and asynchronously executing tasks. The scheduling primitives leverages on the underlying hardware for unparalleled performance and predictability, in effect RTIC provides in Rust terms a zero-cost abstraction to concurrent real-time programming.

## Comparison regarding safety and security

Comparing RTIC to traditional a Real-Time Operating System (RTOS) is hard. Firstly, a traditional RTOS typically comes with no guarantees regarding system safety, even the most hardened kernels like the formally verified [seL4] kernel. Their claims to integrity, confidentiality, and availability regards only the kernel itself (under additional assumptions its configuration and environment). They even state: 

"An OS kernel, verified or not, does not automatically make a system secure. In fact, any system, no matter how secure, can be used in insecure ways." - [seL4 FAQ][sel4faq]

[sel4faq]: https://docs.sel4.systems/projects/sel4/frequently-asked-questions.html

[seL4]: https://sel4.systems/

## Security by design 

In the world of information security we commonly find:

- confidentiality, protecting the information from being exposed to an unauthorized party, 
- integrity, referring to accuracy and completeness of data, and
- availability, referring to data being accessible to authorized users.

Obviously, a traditional OS can guarantee neither confidentiality nor integrity, as both requires the security critical code to be trusted. Regarding availability, this typically boils down to the usage of system resources. Any OS that allows for dynamic allocation of resources, relies on that the application correctly handles allocations/de-allocations, and cases of allocation failures.  

Thus their claim is correct, security is completely out of hands for the OS, the best we can hope for is that it does not add further vulnerabilities.

RTIC on the other hand holds your back. The declarative system wide model gives you a static set of tasks and resources, with precise control over what data is shared and between which parties. Moreover, Rust as a programming language comes with strong properties regarding integrity (compile time aliasing, mutability and lifetime guarantees, together with ensured data validity).

Using RTIC these properties propagate to the system wide model, without interference of other applications running. The RTIC kernel is internally infallible without any need of dynamically allocated data. 
