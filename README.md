# server-web
Web front server

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

## Basic signup/signin

Accounts can be created and signed into using `/user/signup` and `/user/signin` routes respectively

## TODO

- client request abstraction
- session abstraction
- extract variables into .env config
- don't create student subscription before manual review of student ID
- keep track whether a subscription is active/paid
- redirect to stripe payment page if not active
- handle stripe error responses more comprehensively
- consolidate forms, make backend the source of truth to generate form html and validation
- database encryption
- refactor into DDD
- change emails to use handlebars renderer and interpolation
- extract internal urls into env variables
- test billing
- check accept-language header for preferred language
- implement job scheduler for token cleanup and subscription verification
- wrap reviews when last review is reached
