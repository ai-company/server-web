# server-web
Web front server

## TODO
- client request abstraction
- session abstraction
- extract variables into .env config



# Auth module

A simple tool to handle Authentication, exposing REST endpoints to:

- Create/Verify users
- (soon) view subscription status

## Getting started

Create a file in the `/src` folder named `keys.rs`. This file contains all the service keys so should never be commited
to git EVER. This is why you need to make your own copy with the following:

```rust
pub const STRIPE_KEY: &str = /* KEY FOR STRIPE */;

pub const STRIPE_WEBHOOK_SECRET: &str = /* SECRET FOR STRIPE WEBHOOK */;
```

## Structure

All endpoints for the auth api is in the `/src/route` folder, where folders/files correspond to their path in the api.
So `/src/route/payment/stripe/checkout` is `/payment/stripe/checkout` and `/src/route/token/main` is `/token/`

## Database

For the database the `initialize.rs` file is used for table creation/migration, this works by using the `user_version`
variable in any Sqlite database, where 0 is assumed to be unintialized, and any other to be the current scheme version.
