
# Machine Readable Spec for WGSL (WebGPU Shading Language)

This repo contains scripts for scraping the [WGSL
spec](https://www.w3.org/TR/WGSL/) and produces a set of JSON files in the
`spec/` directory:

- `spec/tokens.json`: keywords and special names.
- `spec/functions.json`: information about builtin functions, complete with signatures and documentation.


## Building

You can update the spec by running

```
./x update-upstream
```

And then to regenerate the bindings:

```
./x run
```
