preflight-example action:
    cargo build --bin cargo-preflight
    cd example && cargo preflight {{action}}