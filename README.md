# RUID - Time-Travel-Safe Unique 64 bit ids generated in Rust

RUIDs (Rodrigo's Unique Identifiers) are 64 bit ids mathematically guaranteed to be unique when generated within the same _RUID root_.

An _RUID root_ is a set of RUID generators where each generator can be uniquely identified through shared configuration. E.g. a root can be implemented as a set of VMs on the same subnet, each identified by the last n bits of its internal IP address.

# Schema Design

The canonical version of RUIDs (this repo) uses 41 bits for timestamp, 14 bits for a monotonically increasing sequence, and 9 bits for the root id.

- 41 bits is enough to cover Rodrigo's projected lifespan in milliseconds.
- 14 bits is about the # of RUIDs that can be generated single threaded in Rodrigo's personal computer (~20M ids per second).
- 9 bits is what remains after the calculations above, and is used for root id. The root id is further split into 5 bits for a cluster id, and 4 bits for a node id.

# Time Travel

RUIDs are designed with time travel as a requirement. Whereas other unique id implementations fail (sometimes silently) if the system generating ids goes back in time, RUIDs will still output valid, unique ids.

In v0.1, this is achieved by:

- Defining a millisecond maximum time travel threshold `MMTTT` (sometimes shortened as `M2T3`).
- Comparing the current generation timestamp `Ct` with the previous generation timestamp `Pt`. When `Ct < Ct + MMTTT < Pt`, RUIDs are generated with `Pt` as the timestamp.
- Sleeping for `MMTTT` when the server starts, and validating the system clock indeed increased by at least `MMTTT` at the end.

Note that timestamps for RUIDs generated after time travel and before MMTTT has elapsed will not match the system's clock, which is both a feature and a bug (unsurprisingly, time travel incurs bug/feature duality).

Unfortunately this design is not mathematically correct if time travel happens while the RUID generator is not running; plans for fixing this bug--technically a higgs-bugson--are underway and planned for a v2 release of RUID.

# Performance

Being coded in Rust and statically linked to musl, the RUID generator is exceptionally performant. v0.1 provides RUIDs via an actix HTTP server, for ease of integration and testing. The resulting standalone docker container is less than 15MB uncompressed. Further optimizations can be made by moving to a more performant RPC framework, and are planned for the RUID v1 release.

# Why?

Rodrigo needed unique 64 bit ids to run benchmarks against 128 bit UUIDs in various distributed, database-intensive scenarios. Rodrigo was unsatisfied with existing implementations for various reasons, including questionable programming language choices and flaky project names.

# Inspiration

RUIDs were inspired by the great efforts other engineers have gone through to generate 64 bit application-unique identifiers. In particular, inspiration was drawn from [Instagram](https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c), [Twitter's Snowflake](https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake.html), and [Sony's Sonyflake](https://github.com/sony/sonyflake).
