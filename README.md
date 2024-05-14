# beehive
`beehive` is command line tool for configuring Abeeway devices over BLE.

Before using the tool, you need to pair your device with your Operating System (Windows). 

The tool is using cross-platform libraries so that it can be compiled for Windows, Linux and MacOS. It has been tested on Windows and a compiled Windows-binary file is already available in the `target/release` folder.

Once you have paired your Abeeway device with your Windows OS, you can search for advertizing device as follows: 

```
    beehive -l
```
`beehive` will scan for devices for up to 15s and show the result.
If there are no devices found, you should make sure that your device is adwertizing. On abeeway Microtrackers and Smart Badges, you can trigger advertizements by turning OFF and ON the device again. Once you turn on the tracker it will advertize for a few minutes.

After one or more devices were found, you can connect to one of them them by executing the following command:

```
    beehive --cli <DEVICE>
```
`<DEVICE>` is the name of the device as it was printed by the `beehive -l` command.
Make sure that the tracker is still advertizing, otherwise it cannot be found by your computer.

After you finished your work with the Command Line Interface press `Ctrl-C` to exit.

If later on you want to pair your Abeeway Device with another computer, you need to remove the BLE bond by executing the following command:

```
    beehive --unpair <DEVICE>
```

All usage options of the `beehive` tool are listed below:

```
Usage: beehive.exe <--list|--cli <DEVICE>|--show <DEVICE>>

Options:
  -l, --list             Lists advertizing Abeeway devices
      --show <DEVICE>    Shows details of the selected device
      --cli <DEVICE>     Opens a Command Line Interface for the selected device
      --unpair <DEVICE>  Remove previously set BLE bond
  -d, --debug...         Turn debugging information on
  -h, --help             Print help
  -V, --version          Print version
```
