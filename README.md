# Sunt

SNMP Agent for Linux written in Rust.

# Intro

SNMP is still the common ground for getting data into various kinds of monitoring
systems. Sunt aims to be a modern SNMP agent that is aware of how things are run
nowadays, stripped to the essentials but adding features where they make sense.

# Supported tables:

* hrStorageTable
* dskTable
* diskIOTable
* ifTable

# Example queries

    snmpwalk     -v2c -c test 127.0.0.1 .1
    snmptable    -v2c -c sunt 127.0.0.1 hrStorageTable
    snmptable    -v2c -c sunt 127.0.0.1 dskTable
    snmptable    -v2c -c sunt 127.0.0.1 diskIOTable
    snmptable    -v2c -c sunt 127.0.0.1 ifTable
    snmpbulkwalk -v2c -c test 127.0.0.1 dskTable
    snmpbulkwalk -v2c -c derp 127.0.0.1 .1.3.6.1.2.1.31.1.1.1

# Notable differences to net-snmpd

* No write access whatsoever

* No access control, community string is completely ignored

* hrStorageTable:

    * Only actual mountpoints are included (no RAM etc).
    * Duplicate mountpoints (bind mounts) are filtered out (useful for Docker/LXC hosts).

* diskIOTable:

    * `dm-*` devices are reported as the actual device, e.g. `vghive/data` or `crypted_home`. 

* ifTable

    * Only Physical interfaces, Bridges and VLAN interfaces are exported (VM interfaces and VPN tunnels are filtered).

* Considerably faster response

   Tested using

        time snmptable -v2c -c community host diskIOTable
        time snmptable -v2c -c community host hrStorageTable
        time snmptable -v2c -c community host ifTable
        time snmpbulkwalk -v2c -c community host .1 > /dev/null

   Over a local gigabit connection:

            Table        |   Sunt                 |  net-snmpd
        ---------------- | ---------------------- | --------------------
          diskIOTable    |   real    0m0,028s     |  real    0m0,038s
          (26 entries)   |   user    0m0,019s     |  user    0m0,020s
                         |   sys     0m0,005s     |  sys     0m0,006s
                         |                        |
          hrStorageTable |   real    0m0,028s     |  real    0m0,031s
          (17 entries)   |   user    0m0,020s     |  user    0m0,017s
                         |   sys     0m0,006s     |  sys     0m0,008s
                         |                        |
          ifTable        |   real    0m0,041s     |  real    0m0,088s
          (26 entries)   |   user    0m0,019s     |  user    0m0,027s
                         |   sys     0m0,012s     |  sys     0m0,011s
                         |                        |
          full bulkwalk  |   real    0m0,041s     |  real    0m3,183s
          (start at .1)  |   user    0m0,021s     |  user    0m0,360s
                         |   sys     0m0,008s     |  sys     0m0,105s
                         |                        |

   Over a remote connection with about 20ms latency:

            Table        |   Sunt                 |  net-snmpd
        ---------------- | ---------------------- | --------------------
          diskIOTable    |   real    0m0,103s     |  real    0m0,624s
          (26 entries)   |   user    0m0,026s     |  user    0m0,020s
                         |   sys     0m0,012s     |  sys     0m0,011s
                         |                        |
          hrStorageTable |   real    0m0,057s     |  real    0m0,317s
          (17 entries)   |   user    0m0,025s     |  user    0m0,027s
                         |   sys     0m0,008s     |  sys     0m0,012s
                         |                        |
          ifTable        |   real    0m0,130s     |  real    0m3,474s
          (26 entries)   |   user    0m0,022s     |  user    0m0,044s
                         |   sys     0m0,019s     |  sys     0m0,030s
                         |                        |
          full bulkwalk  |   real    0m0,265s     |  real    0m55,677s
          (start at .1)  |   user    0m0,021s     |  user    0m0,620s
                         |   sys     0m0,007s     |  sys     0m0,253s

   Note that while this test is somewhat unfair because sunt returns fewer data,
   it appears that net-snmp is worse affected by latency.
