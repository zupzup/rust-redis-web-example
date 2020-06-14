# rust-redis-web-example

Example of Using Redis in a Rust Web Application implementing three different ways to use redis within a warp web service:

* [redis-rs without a pool](https://github.com/mitsuhiko/redis-rs) 
* [mobc](https://github.com/importcjj/mobc)
* [r2d2](https://github.com/sorccu/r2d2-redis)

# Setup

To start, run:

```bash
make dev
```

And a local redis instance:

```bash
docker run -p 6379:6379 redis:5.0
```

And then you can test the different endpoints with:

```bash
curl http://localhost:8080/mobc
mobc_world

curl http://localhost:8080/r2d2
r2d2_world

curl http://localhost:8080/direct
direct_world
```
