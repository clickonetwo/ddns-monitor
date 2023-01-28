# Dynamic DNS Monitor

The `ddns-monitor` is a very simple, background process that periodically checks whether Dynamic DNS address assignments for a list of hosts has changed, and sends mail about it if so.  It's meant to be run as a daemon.

This is not a mature, configurable service! The SMTP server to use and the content of the email are built into the code.  The list of hosts, the email addresses, and passwords are taken from environment variables.

# License

All the material in this repository is made freely available under the open-source MIT license.  See the LICENSE file for details.
