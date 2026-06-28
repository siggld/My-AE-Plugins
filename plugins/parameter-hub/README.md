# parameter-hub ( TKG_ParameterHub )

Scaffolds a hub-style controller for collecting and relaying AE effect parameters.

This is the After Effects plugin **TKG_ParameterHub**, which provides the **ParameterHub.aex** plugin file for Adobe After Effects.

## Current scope

This initial scaffold adds the plugin shell, advanced-setting placeholders, and a passthrough render path.

The full `Smart Add` workflow from the specification requires host-side operations such as inspecting selected properties, creating Hub controls, rewriting expressions, and relinking existing parameters. Those capabilities likely need AEGP or script-side support in addition to a regular `AEEffect`.

## Building the Plugin

See the [main README](../../README.md) for instructions on how to build the plugin.
