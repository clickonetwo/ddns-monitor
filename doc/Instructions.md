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
7. Open a terminal session by typing `cmd.exe` in the Start menu search box and hitting return.  In the terminal session give the command `ddns-monitor configure` and answer the configuration questions (see the [Configuration](#configuration) section).
8. Download the [`ddns-monitor-install.cmd`](./ddns-monitor-install.cmd) file from the same directory as these instructions, and put it in the `C:\Program Files\nssm` directory.
9. Edit the `ddns-monitor-install.cmd` file:
   * Replace `<username>` with your Windows account name (4 occurrences);
   * Replace `<login password>` with your Windows login password (1 occurrence).
10. Open a command prompt with elevated privileges, as follows: in the Start menu search box, type `cmd.exe` to find the Windows command prompt program, right click on it, and choose `Run as administrator`.
11. In the command prompt window, give the command `ddns-monitor-install` (followed by the enter key).  You will see commands starting with `nssm` being executed, and (hopefully) no error messages.

At this point, the dynamic DNS monitor service is installed and will run the next time you reboot your computer.

### Mac installation

You can only use `ddns-monitor` on Mac OS 10.12 or newer.  These instructions are for running it as “Launch Daemon,” which means it runs regardless of who is logged in on the computer.

1. In the Terminal, give the command `mkdir -p ~/Applications` to ensure you have a user-owned Applications directory.

2. In your web browser, download the appropriate Mac build (`x86_64` for an Intel machine, `aarm64` for an Apple Silicon machine) from the [latest release of ddns-monitor](https://github.com/clickonetwo/ddns-monitor/releases/latest). 

3. In the Terminal, give these two commands, the first of which moves the program from where it was downloaded to where it will be used, and the second of which makes it executable:

   ```shell
   mv ~/Downloads/ddns-monitor.* ~/Applications/ddns-monitor
   chmod a+x ~/Applications
   ```

4. In the Terminal, run the monitor to configure it: `~/Applications/ddns-monitor configure` (see the [Configuration](#configuration) section).

5. Download the [`io.ClickOneTwo.ddns-monitor.plist`](./io.ClickOneTwo.ddns-monitor.plist) file from the same directory as these instructions.

6. Edit the downloaded copy of the `io.ClickOneTwo.ddns-monitor.plist` file:

   - Replace `USERNAME` (all caps, 4 occurrences) with your Mac account name (which is the name of your home directory, not your full name).

7. In the terminal, move your edited copy of the plist file into place with the command:

   ```shell
   sudo mv ~/Downloads/io.ClickOneTwo.ddns-monitor.plist /Library/LaunchDaemons/
   ```

At this point, the Dynamic DNS monitor service is installed and will run the next time you reboot your computer.

### Linux installation

You can probably use `ddns-monitor` on any Linux system, but the premade build is for Ubuntu so those are the instructions we give here.  On other architectures you may need to adjust them.

1. In a terminal, give the command `mkdir -p ~/bin` to ensure you have a user-owned binaries directory.

2. In your web browser, download the Ubuntu build from the [latest release of ddns-monitor](https://github.com/clickonetwo/ddns-monitor/releases/latest). 

3. In a terminal, give these two commands, the first of which moves the program from where it was downloaded to where it will be used, and the second of which makes it executable:

   ```shell
   mv ~/Downloads/ddns-monitor.* ~/Applications/ddns-monitor
   chmod a+x ~/bin
   ```

4. In a terminal, run the monitor to configure it: `~/bin/ddns-monitor configure` (see the [Configuration](#configuration) section).

5. Download the [`ddns-monitor.service`](./ddns-monitor.service) file from the same directory as these instructions.

6. Edit the downloaded copy of the `ddns-monitor.service` file:

   - Replace `USERNAME` (all caps, 5 occurrences) with your Linux login name (which is the name of your home directory, not your full name).

7. In the terminal, move your edited copy of the service file into place, and register the service with the system, using these two commands:

   ```shell
   sudo mv ~/Downloads/ddns-monitor.service /etc/systemd/system/
   sudo sudo systemctl enable ddns-monitor
   ```

At this point, the Dynamic DNS monitor service is installed and will run the next time you reboot your computer.

## Configuration

The configuration instructions for `ddns-monitor` are the same for all platforms.  You *must* configure it before running it as a monitoring service.  If it’s launched as a service without being configured, it will log an error message and then quit.

To configure `ddns-monitor`, launch it in a terminal session with an argument of `configure`.  It will interview you in the terminal to collect the following configuration information:

1. The SMTP (email) server that you use to send mail, such as `smtp.gmail.com` for Google-hosted addresses.  This is the same server you use with your mail program.
2. The `From` email address/server login account that you use to send emails.
3. The password for your server account. This password is stored encrypted in the configuration, and the configuration is only readable by your account, so there’s no risk in entering the password.  (_N.B._ If you use Google mail, and you have disallowed “less secure access,” you will need to have created a Google `application password` for use by `ddns-monitor`.)
4. The `To` email addresses that you want notifications to be sent to.  This can include yourself.
5. The hostnames (DNS names) that you want monitored to see if their IP address changes.
