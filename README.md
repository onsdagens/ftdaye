# ftdaye

Experiment with low-level jtag functionality.

The code is based on the internal jtag support within the [probe-rs](https://github.com/probe-rs/probe-rs) project as an experiment to if modularization and re-use can be improved.

## openocd

The openocd script reads the IDCODE, (0362d093 for the ARTY Artix7 T-35).

```shell
openocd -f openocd_read.cfg
```
