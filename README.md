# beehive

`beehive` is command line tool for configuring Abeeway devices over BLE.

Before using the tool, you need to pair your device with your Operating System (Windows or Linux).

The tool is using cross-platform libraries so that it can be compiled for Windows, Linux and MacOS. It has been tested and compiled for Windows and Linux. The pre-compiled binary files already available in the `target/release` folder (`beehive` for Linix, `beehive.exe` for Windows).

Once you have paired your Abeeway device with your Windows/Linux OS, you can search for advertizing devices as follows:

```bash
    beehive scan
```

*(Run the above comman in Windows PowerShell or in a Linux shell, e.g. bash.)*

`beehive` will scan for devices for up to `10s` and show the result.
If there are no devices found, you should make sure that your device is advertizing. You can trigger advertizements on Abeeway Microtrackers and Smart Badges by turning them OFF and ON again. Once you turn a tracker ON, they will advertize for a few minutes. On Abeeway Compact trackers you can trigger advertizements several times by placing and removing a magnet to/from their marked sides.

After one or more devices were found, you can connect to one of them by executing the following command:

```bash
    beehive cli <DEVICE>
```

`<DEVICE>` is the device's name as it was printed by the `beehive scan` command.
Make sure that the tracker is still advertizing, otherwise it cannot be found by your computer.

After you finished your work with the Command Line Interface press `Ctrl-C` to exit.

If later on you want to pair your Abeeway Device with another computer, you need to remove the BLE bond by executing the following command:

```bash
    beehive remove-bond <DEVICE>
```

It is important to note, that you can remove the BLE bond only from that computer that the bond was set on earlier.

All usage options of the `beehive` tool are listed below:

```bash
Usage: beehive.exe [OPTIONS] [COMMAND]

Commands:
  scan              Scan for Abeeway devices.
  show              Show device details.
  cli               Open Command Line Interface.
  remove-bond       Remove BLE bond.
  export-config     Export configuration.
  import-config     Import configuration.
  firmware-upgrade  COMMING SOON - Upgrade MCU firmware.
  help              Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Show logs for debugging (-v|-vv|-vvv)
  -h, --help        Print help
  -V, --version     Print version
```
