fly -t cutters validate-pipeline -c pipelines/bookstore.yml -l pipelines/vars.yml --output

fly -t bookstore trigger-job -j bookstore/lint --watch
fly -t bookstore execute -c manifests/pipelines/tasks/test.yml  -i bookstore=.
fly -t bookstore intercept -j bookstore/lint

Highlight composability
Show you can override vars in fly execute

Show running tests
Introduce an error
Show running a single test
Show:
    RUST_LOG=sqlx::query cargo test book_registering
    RUST_LOG=[request] cargo test book_registering
fly -t cutters watch -j bookstore/lint
