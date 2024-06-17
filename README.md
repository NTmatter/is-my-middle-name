# is-my-middle-name (IMMN)
[is-my-middle.name](https://lambda.is-my-middle.name) (IMMN) is a bespoke redirect service for individuals with notable middle names.

Architecturally, IMMN is a single Lambda function that matches on hostname (or request uri) and returns a 403 redirect or other content where needed.

# Development
IMMN is built on [Cargo Lambda](https://cargo-lambda.info) for simplicity and intersection of trendy technologies.

Start a local preview environment with:
```shell
cargo lambda watch
```

After a brief build, the lambda function will be available at http://localhost:9000/
