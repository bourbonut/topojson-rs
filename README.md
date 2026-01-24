# Topojson Client in Rust

Implementation of [`topojson-client`](https://github.com/topojson/topojson-client) in Rust for creating native Python extension.

- [Documentation](https://topojson-rs.readthedocs.io/en/latest/)

## Installation

```bash
pip install topojson_rs
```

## Development

1. You need [`cargo`](https://rust-lang.org/tools/install/) installed and [`maturin`](https://pypi.org/project/maturin/).
2. Create a virtual environment
3. Build the package:
```
maturin develop --release
```
4. You should be able to import `topojson`.

### Run tests

```bash
cargo test
```

## Benchmarks

Current benchmark (see `benchmark.py`, files `land`, `states` and `counties` available [here](https://github.com/topojson/us-atlas) and [fork of `pytopojson`](https://github.com/bourbonut/pytopojson) for correct results).

<table>
  <tr>
    <td></td>
    <td colspan=3>Reading performances</td>
    <td colspan=3>Computation performances</td>
    <td></td>
  </tr>
  <tr>
    <td>Function + Data</td>
    <td>ratio</td>
    <td>python</td>
    <td>rust</td>
    <td>ratio</td>
    <td>python</td>
    <td>rust</td>
    <td>Same ?</td>
  </tr>
  <tr>
    <td>Feature Land</td>
    <td>2.499</td>
    <td>1.359 ms</td>
    <td>0.544 ms</td>
    <td>19.277</td>
    <td>2.154 ms</td>
    <td>0.112 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Feature States</td>
    <td>2.163</td>
    <td>2.856 ms</td>
    <td>1.320 ms</td>
    <td>41.704</td>
    <td>6.581 ms</td>
    <td>0.158 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Feature Counties</td>
    <td>1.729</td>
    <td>19.520 ms</td>
    <td>11.289 ms</td>
    <td>39.676</td>
    <td>87.367 ms</td>
    <td>2.202 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Mesh Land</td>
    <td>2.234</td>
    <td>1.149 ms</td>
    <td>0.514 ms</td>
    <td>18.909</td>
    <td>2.713 ms</td>
    <td>0.143 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Mesh States</td>
    <td>1.914</td>
    <td>2.529 ms</td>
    <td>1.321 ms</td>
    <td>16.568</td>
    <td>6.325 ms</td>
    <td>0.382 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Mesh Counties</td>
    <td>1.473</td>
    <td>16.426 ms</td>
    <td>11.148 ms</td>
    <td>10.043</td>
    <td>87.903 ms</td>
    <td>8.753 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Merge Land</td>
    <td>2.096</td>
    <td>1.195 ms</td>
    <td>0.570 ms</td>
    <td>18.816</td>
    <td>3.711 ms</td>
    <td>0.197 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Merge States</td>
    <td>1.923</td>
    <td>2.631 ms</td>
    <td>1.368 ms</td>
    <td>13.350</td>
    <td>5.605 ms</td>
    <td>0.420 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Merge Counties</td>
    <td>1.432</td>
    <td>15.946 ms</td>
    <td>11.133 ms</td>
    <td>7.546</td>
    <td>33.093 ms</td>
    <td>4.385 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Bbox Land</td>
    <td>1.980</td>
    <td>3.121 ms</td>
    <td>1.576 ms</td>
    <td>27.834</td>
    <td>2.555 ms</td>
    <td>0.092 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Bbox States</td>
    <td>1.145</td>
    <td>5.276 ms</td>
    <td>4.608 ms</td>
    <td>25.082</td>
    <td>5.070 ms</td>
    <td>0.202 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Bbox Counties</td>
    <td>1.556</td>
    <td>16.742 ms</td>
    <td>10.759 ms</td>
    <td>61.178</td>
    <td>15.506 ms</td>
    <td>0.253 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Neighbors Land</td>
    <td>2.354</td>
    <td>1.197 ms</td>
    <td>0.508 ms</td>
    <td>0.798</td>
    <td>0.071 ms</td>
    <td>0.089 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Neighbors States</td>
    <td>1.957</td>
    <td>2.636 ms</td>
    <td>1.347 ms</td>
    <td>2.704</td>
    <td>0.558 ms</td>
    <td>0.206 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Neighbors Counties</td>
    <td>1.515</td>
    <td>16.174 ms</td>
    <td>10.677 ms</td>
    <td>5.033</td>
    <td>20.601 ms</td>
    <td>4.093 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Quantize Land</td>
    <td>2.359</td>
    <td>1.201 ms</td>
    <td>0.509 ms</td>
    <td>30.167</td>
    <td>2.851 ms</td>
    <td>0.095 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Quantize States</td>
    <td>1.861</td>
    <td>2.492 ms</td>
    <td>1.339 ms</td>
    <td>27.742</td>
    <td>7.166 ms</td>
    <td>0.258 ms</td>
    <td>True</td>
  </tr>
  <tr>
    <td>Quantize Counties</td>
    <td>1.711</td>
    <td>16.913 ms</td>
    <td>9.883 ms</td>
    <td>20.389</td>
    <td>48.865 ms</td>
    <td>2.397 ms</td>
    <td>True</td>
  </tr>
</table>
