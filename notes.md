helm repo add concourse https://concourse-charts.storage.googleapis.com/
helm upgrade -f myvalues.yaml concourse concourse/concourse

helm repo add keel https://charts.keel.sh
helm repo update


fly -t tutorial validate-pipeline -c manifests/pipelines/bookstore.yml -l manifests/pipelines/vars.yml --output

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


