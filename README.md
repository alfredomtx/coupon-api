# 🦀 What is this project?

This is a new personal project I'm working on, a Coupon API for [OldBot](http://oldbot.com.br/)(my other project) where I validate coupons and license discounts.

The main technologies and features are:

- [Rust](https://www.rust-lang.org/)
- [Actix Web](https://actix.rs/) framework
- [Tokio](https://tokio.rs/) as asynchronous runtime
- [SQLx](https://github.com/launchbadge/sqlx) as "kind of" the ORM
- [MySQL](https://www.mysql.com/) database
- [Redis](https://redis.com/) for caching and storing sessions
- [Docker](https://www.docker.com/) container
- Simple `Bearer` authentication and session validation
- Unit and integration test for all API endpoints

The application has been deployed on [Heroku](https://www.heroku.com/) in a Docker container, and is live at https://coupon-api-oldbot.herokuapp.com/.

Since it requires authentication, you won't be able to interact with it. I will work on a demo version of it where others can interact with it in a test database in the future.

### Postman

In this repository, you can also find the `Coupon API.postman_collection.json` file, which you can import on [Postman](https://www.postman.com/) to have a template for the API calls of all endpoints available.

![image](https://user-images.githubusercontent.com/20379136/210445498-da776304-5ea9-4a08-b6b8-31763de92fd0.png)
