# Instructions

These are the instructions for installing and configuring the Dynamic DNS Monitor.

## Installation

The installation instructions for `ddns-monitor` depend on your platform (of course).

### Windows installation

You can only use `nssm` on Windows 10 or higher.  Because Windows services are tricky, our recommendation is to install `ddns-monitor` as an [`nssm`](https://nssm.cc)-based service.  Follow these steps:

1. Create the directory `C:\Program Files\nssm` in the Windows `File Explorer` app.  (This will require administrative privileges.)
2. Add the just-created directory `C:\Program Files\nssm` to the *System* `PATH` variable (following [these instructions](https://windowsloop.com/how-to-add-to-windows-path/) or others you can find on the web).
3. Download the [2.24 release of `nssm`](https://nssm.cc/release/nssm-2.24.zip).
4. Unzip the release and put the `nssm.exe` program in the `C:\Program Files\nssm` directory.
5. Download the Windows build from the [latest release of ddns-monitor](https://github.com/clickonetwo/ddns-monitor/releases/latest).
6. Move it to the `C:\Program Files\nssm` directory and rename it `ddns-monitor.exe`.
7. Double-click the `ddns-monitor.exe` file to launch it.  A terminal window will open and interview you to collect configuration information (see the [Configuration](#configuration) section).
8. Download the [ddns-monitor-install.cmd](./ddns-monitor-install.cmd) file from the same directory as these instructions, and put it in the `C:\Program Files\nssm` directory.
9. Edit the `ddns-monitor-install.cmd` file and:
   * replace `<username>` with your Windows account name (4 occurrences);
   * Replace `<login password>` with your Windows login password (1 occurrence).
10. Open a command prompt with elevated privileges, as follows: in the Start menu search box, type `cmd.exe` to find the Windows command prompt program, right click on it, and choose `Run as administrator`.
11. In the command prompt window, give the command `ddns-monitor-install` (followed by the enter key).

At this point, the dynamic DNS monitor service is installed and will run the next time you reboot your computer.

## Configuration

The configuration instructions for `ddns-monitor` are the same for all platforms.  You *must* configure it before running it as a monitoring service.  If it’s launched as a service without being configured, it will log an error message and then quit.

To configure `ddns-monitor`, launch it in a terminal session with an argument of `configure`.  It will interview you in the terminal to collect the following configuration information:

1. The SMTP (email) server that you use to send mail, such as `smtp.gmail.com` for Google-hosted addresses.  This is the same server you use with your mail program.
2. The `From` email address/server login account that you use to send emails.
3. The password for your server account. This password is stored encrypted in the configuration, and the configuration is only readable by your account, so there’s no risk in entering the password.  (_N.B._ If you use Gmail, and you have disallowed “less secure access,” you will need to have created a Google `application password` for use by `ddns-monitor`.)
4. The `To` email addresses that you want notifications to be sent to.  This can include yourself.
5. The hostnames (DNS names) that you want monitored to see if their IP address changes.