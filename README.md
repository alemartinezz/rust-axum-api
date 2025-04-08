# Hola

Custom max body size for specific route

```
Router::new()
    .route(
        "/upload",
        post(upload).layer(DefaultBodyLimit::max(52_428_800)),
    )
    .route("/other-route", get(foobar))
```