# How the Stripe flow occurs for the user

1. They create an Orto account
2. They pick their account type, and then send a request to our backend to create a "Checkout", this checkout should also include the `price_id` describing the exact subscription the customer wants to sign up for.

   1. The backend then calls the `stripe_profile::get_or_create` function, which returns a stripe profile for this user, either by creating a new one on stripe using the `stripe::customer::create_customer` function, or reading from the database. (Note this should probably be fixed so we also check the stripe service for an exising customer with an email in case of data loss from the database)

   2. The backend then sends a POST request to Stripe with a shared secret as authorization, saying user with id X wants to see a checkout with a certain payment options, and URL's to visit on success or failure, and the users cart should contain a given amount of subscriptions (In our case one of 3).

   3. If a Checkout is created the backend returns the ID of this checkout to the frontend.

3. On the frontend the client invokes `redirectToCheckout` on the Stripe JS library with the Checkout id to navigate to the checkout page. (This might be possible without the Stripe lib, haven't looked into it)
4. The user ends up on the Stripe hosted checkout page, and hopefully completes the transaction.
5. Stripe sends a webhook to our backend informing it of the new subscription, and redirects the user to the SUCCESS url.
