# 🦀 What is this project?
This is a new personal project I'm working on, a Coupon API for [OldBot](http://oldbot.com.br/)(my other project) where I will validate coupons and license discounts.

The main technologies used are:
- [Rust](https://www.rust-lang.org/).
- [Actix Web](https://actix.rs/) framework.
- [Tokio](https://tokio.rs/) as asynchronous runtime.
- [SQLx](https://github.com/launchbadge/sqlx)
- [MySQL](https://www.mysql.com/) database.
- Simple Bearer token authentication.
- [Docker](https://www.docker.com/) container.

It has been deployed on Heroku in a Docker container and is live at https://coupon-api-oldbot.herokuapp.com/, since it requires authentication, you won't be able to interact with it. I will work on a demo version of it where others can interact with it in a test database.

In this repository, you can also find the `Coupon API.postman_collection.json` file, which you can import on Postman to have a template for the API calls.
