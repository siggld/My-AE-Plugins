# Parameter Hub MVP architecture notes

## Summary

The attached specification describes a Hub workflow that goes beyond a normal image-processing effect. In particular, these operations are outside the usual `AEEffect` comfort zone and should be treated as a separate integration track:

- reading the user's currently selected arbitrary project property
- creating or locating a `__Parameter Hub` layer in the target comp
- creating proxy controls that match the source property type
- copying keyframes and expressions from the source property into the Hub proxy
- rewriting context-dependent expressions before relinking the source property

## Recommended implementation split

### Track A: effect-side UI and host-visible state

Keep `TKG_ParameterHub` as the visible effect shell that exposes:

- user-facing options from the specification
- status display for current limitations and future link health
- optional custom UI for browsing registered items later

### Track B: host/project manipulation layer

Implement the actual `Smart Add` flow with a host-manipulation path that can:

- inspect selected properties
- create or find Hub layers and controls
- set expressions on external properties
- migrate keyframes and expression strings safely

This track may need AEGP suites, a companion script, or both. The correct path should be validated before the first functional MVP implementation.

## MVP boundary for the next coding step

The first functional milestone should stay narrow:

1. Limit support to `Slider / OneD`.
2. Target only properties inside the active comp.
3. Create or find `__Parameter Hub`.
4. Create one proxy control with a deterministic name.
5. Copy the current value only.
6. Set the source property expression to reference the Hub control.

After that works reliably, add keyframe transfer, expression transfer, and list/status UI.
