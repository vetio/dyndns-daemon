# dyndns-daemon

Implements a dynamic DNS service for Hetzner's Domain Registration Robot.

Developed for the FRITZ!Box 7362 SL.

## Requirements

- gpg

## Configuration

dyndns-daemon reads its configuration from the environment. Necessary entries are:

| Name | Description | Type |
| --- | --- | --- |
| FROM_ADDR | "From" address as used in the email to the robot. | String |
| TO_ADDR | Address of the robot. (robot@robot.first-ns.de) | String |
| SMTP_HOST | Hostname of the SMTP server | String |
| SMTP_PORT | Port for the SMTP service | U16 (0-65535) |
| SMTP_USERNAME | Username for the SMTP service | String
| SMTP_PASSWORD | Password for thr SMTP service | String |
| PGP_KEY | ID of the GPG Key that will be used for signing the email | String |
| DOMAIN | Domain that will be managed | String |
| HETZNER_USER | Hetzner username | String |
| SERVER_ADDR | Address on which will be listened for HTTP requests | String |
| HTTP_AUTH_USER | Username for HTTP authentication for incoming requests. | String |
| HTTP_AUTH_PASSWORD | Password for HTTP authentication for incoming requests. |  String |
| IP_HEADER | Name of the header which contains the IP address of the true client. | String |
| TEMPLATE | File containing a template for the generated zonefile.| String |

See also the [exmaple .env file](res/config.env).

## Template

To generate a zonefile for the managed domain, dyndns-daemon uses a template, where `{%SERIAL%}` and `{%IP%}` are 
replaced by a 64-bit timestamp and the client ip respectively.

```
@ IN SOA ns1.first-ns.de. postmaster.robot.first-ns.de. (
        {%SERIAL%}; Serial
        86400; Refresh
        7200; Retry
        604800; Expire
        7200); Minimum
@ IN NS ns1.first-ns.de.
@ IN NS ns.second-ns.de.
@ IN A {%IP%}
```

See also the [example file](res/zonefile.tpl).

## Notes

- The GPG key used for signing the email content must not be protected with a password. This is due to gpg refusing to accept the password as an argument and creates a prompt..
