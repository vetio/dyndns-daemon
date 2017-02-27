$TTL 5m
@ IN SOA ns1.first-ns.de. postmaster.robot.first-ns.de. (
        {%SERIAL%}; Serial
        86400; Refresh
        7200; Retry
        604800; Expire
        7200); Minimum
@ IN NS ns1.first-ns.de.
@ IN NS ns.second-ns.de.
@ IN A {%IP%}
