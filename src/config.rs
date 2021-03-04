// We want an epoch that is safe for any code Rodrigo writes.
//
// Future-proofing, in a literal sense, requires us to account for time
// travelling, which is np-hard (google Time-Travelling Salesman Problem).
//
// Fortunately, having watched Umbrella Academy, Rodrigo is aware of the perils
// involved in such endeavours--he wants nothing to do with Time-Travelling
// himself. This makes the Day Rodrigo Learned to Code (DRLC) an ideal epoch:
// Rodrigo will be safe if any time-travelling salesman gives him this binary
// in the past.
//
// This epoch choice has a flaw in that a salesman could sway Rodrigo to code
// sooner, however, given Rodrigo's stubborness, that is highly unlikely. We
// thus mitigate that flaw via the Ostrich Algorithm.
//
pub static DRLC: u64 = 1201737600000;

// Millisecond Maximum Time Travel Threshold
pub static MMTTT: u64 = 1000;

// 69.68 years in milliseconds. This covers Rodrigo's lifespan with DRLC epoch.
pub static TIMESTAMP_BITS: u8 = 41;
pub static MAX_TIMESTAMP: u128 = (1 << TIMESTAMP_BITS) - 1;

// Accept up to 32 clusters.
pub static CLUSTER_BITS: u8 = 5;
pub static MAX_CLUSTER: u64 = (1 << CLUSTER_BITS) - 1;

// 14.43 bits sequence is enough for sequence to not overflow in Rodrigo's
// computer. Cloud machines won't be as fast and will have extra overhead
// so we round down to 14. That leaves 4 bits for Node id. However, given
// we're more constrained by Node # than Sequence #
pub static NODE_BITS: u8 = 4;
pub static SEQUENCE_BITS: u8 = 64 - CLUSTER_BITS - NODE_BITS - TIMESTAMP_BITS;
pub static MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;
pub static TIMESTAMP_SHIFT: u8 = SEQUENCE_BITS + CLUSTER_BITS + NODE_BITS;
pub static SEQUENCE_SHIFT: u8 = CLUSTER_BITS + NODE_BITS;

macro_rules! node_ip_uri {
    () => {
        if cfg!(feature = "local") {
            "http://icanhazip.com/"
        } else if cfg!(feature = "aws-ec2") {
            "http://169.254.169.254/latest/meta-data/local-ipv4"
        } else {
            panic!("node_ip_uri not set")
        }
    };
}
