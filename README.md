# vincenty-rs

### Overview

Expose API endpoint for calculating distance using vincenty formulae.

Notably, it allows you to input `src` and `dst` as query parameters, where either can be a permutation of:

1. latitude,longitude (as floating points)
2. [H3](https://github.com/uber/h3) indices as `String`
3. [H3](https://github.com/uber/h3) indices as `u64`

Check the examples for more usage information.

### Usage

Start the [tide](https://crates.io/crates/tide) server with `cargo run --release`

#### API Example

- With lat,lng as `src` and `dst`:

```shell
$ http POST http://localhost:5000/distance\?src\="42.3541165,-71.0693514"\&dst\="40.7791472, -73.9680804" --body
{
    "data": {
        "distance": 298.396186,
        "dst": {
            "lat": 40.7791472,
            "lng": -73.9680804
        },
        "src": {
            "lat": 42.3541165,
            "lng": -71.0693514
        }
    }
}
```

- With h3 string indices as `src` and `dst`:

```shell
$ http POST http://localhost:5000/distance\?src\="8c2a306638701ff"\&dst\="8c2a100894435ff" --body
{
    "data": {
        "distance": 298.394459,
        "dst": {
            "lat": 40.7791617074961,
            "lng": -73.96812482164383
        },
        "src": {
            "lat": 42.35408591768187,
            "lng": -71.06937831049969
        }
    }
}
```

- With h3 `u64` indices as `src` and `dst`:

```shell
$ http POST http://localhost:5000/distance\?src\=631246145620214271\&dst\=631243921460311551 --body
{
    "data": {
        "distance": 298.394459,
        "dst": {
            "lat": 40.7791617074961,
            "lng": -73.96812482164383
        },
        "src": {
            "lat": 42.35408591768187,
            "lng": -71.06937831049969
        }
    }
}
```

- With a mix of h3 string and h3 `u64`:

```shell
$ http POST http://localhost:5000/distance\?src\="8c2a306638701ff"\&dst\=631243921460311551 --body
{
    "data": {
        "distance": 298.394459,
        "dst": {
            "lat": 40.7791617074961,
            "lng": -73.96812482164383
        },
        "src": {
            "lat": 42.35408591768187,
            "lng": -71.06937831049969
        }
    }
}
```

- With a mix of h3 `u64` and lat,lng:

```shell
$ http POST http://localhost:5000/distance\?src\="42.3541165,-71.0693514"\&dst\=631243921460311551 --body
{
    "data": {
        "distance": 298.398222,
        "dst": {
            "lat": 40.7791617074961,
            "lng": -73.96812482164383
        },
        "src": {
            "lat": 42.3541165,
            "lng": -71.0693514
        }
    }
}
```
