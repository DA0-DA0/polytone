build:
    cargo build

test:
    cargo test

optimize:
    ./devtools/optimize.sh

simtest: optimize
    go clean -testcache
    cd tests/simtests && go test ./...

integrationtest: optimize
	go clean -testcache
	cd tests/strangelove && go test ./...

# ${f    <-- from variable f
#   ##   <-- greedy front trim
#   *    <-- matches anything
#   /    <-- until the last '/'
#  }
# <https://stackoverflow.com/a/3162500>
schema:
    start=$(pwd); \
    for f in ./contracts/*; \
    do \
    echo "generating schema for ${f##*/}"; \
    cd "$f" && cargo schema && cd "$start" \
    ;done
    for f in ./accessories/*; \
    do \
    echo "generating schema for ${f##*/}"; \
    cd "$f" && cargo schema && cd "$start" \
    ;done
