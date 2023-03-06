nssm install ddns-monitor "C:\Program Files\ClickOneTwo\ddns-monitor.exe"
nssm set ddns-monitor AppParameters monitor
nssm set ddns-monitor AppDirectory C:\Users\<username>
nssm set ddns-monitor AppExit Default Exit
nssm set ddns-monitor AppNoConsole 1
nssm set ddns-monitor AppStdout C:\Users\<username>\AppData\Local\ClickOneTwo\ddns-monitor\monitor.log
nssm set ddns-monitor AppStderr C:\Users\<username>\AppData\Local\ClickOneTwo\ddns-monitor\monitor.err
nssm set ddns-monitor AppRotateFiles 1
nssm set ddns-monitor AppRotateOnline 1
nssm set ddns-monitor AppRotateBytes 100000000
nssm set ddns-monitor Description "Monitor dynamic DNS names for changes in IP address"
nssm set ddns-monitor DisplayName "Dynamic DNS Monitor"
nssm set ddns-monitor ObjectName .\<username> "<login-password>"
nssm set ddns-monitor Start SERVICE_AUTO_START
nssm set ddns-monitor Type SERVICE_WIN32_OWN_PROCESS
