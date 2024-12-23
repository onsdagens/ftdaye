# ftdaye

Experiment with low-level jtag functionality.

The code is based on the internal jtag support within the [probe-rs](https://github.com/probe-rs/probe-rs) project as an experiment to if modularization and re-use can be improved.

## Resources

- [ug470](https://docs.amd.com/v/u/en-US/ug470_7Series_Config)

## openocd

The openocd script reads the IDCODE IR 0x09, which contains 0x0362d093 for the ARTY Artix7 T-35.

```shell
openocd -f openocd_read.cfg
```

## examples

The IDCODE can be read using the `ftdaye` API. The IDCODE is loaded to the DR on TLR (reset).

```shell
RUST_LOG=ftdaye=debug,idcode=debug cargo run --example idcode
```

Alternatively after reset we can walk the JTAG STM, setup the IR (0x09, 0_1001 5-bits), and shift out the DR.
The IDCODE value will be loaded to DR once the IR is set, and directly available to the shift register.

Beware, lot's of debug info, select debug level to `warn` if too verbose.
