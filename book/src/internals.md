# Under the hood

This section describes the internals of the RTFM framework at a *high level*.
Low level details like the parsing and code generation done by the procedural
macro (`#[app]`) will not be explained here. The focus will be the analysis of
the user specification and the data structures used by the runtime.
