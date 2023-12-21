#!/usr/bin/env sh
set -eu

SQLITE3_BIN=${SQLITE3_BIN:-sqlite3}
EXTENSION_FILE=${EXTENSION_FILE:-target/debug/examples/libsqlite_compressions}

if [ ! -f "$EXTENSION_FILE" ] && [ ! -f "$EXTENSION_FILE.so" ] && [ ! -f "$EXTENSION_FILE.dylib" ] && [ ! -f "$EXTENSION_FILE.dll" ]; then
    echo "Extension file $EXTENSION_FILE [.so|.dylib|.dll] do not exist. Run 'just build-ext' first. Available files:"
    ls -l $EXTENSION_FILE*
    exit 1
fi
echo "Using extension file '$EXTENSION_FILE [.so|.dylib|.dll]'"

if [ ! command -v $SQLITE3_BIN &> /dev/null ]; then
    echo "$SQLITE3_BIN executable could not be found"
    exit 1
fi
echo "Found $SQLITE3_BIN executable $($SQLITE3_BIN --version)"

test_one() {
    local sql=$1
    local expected=$2

    echo "Trying to get  '$expected'  from  $sql"
    result=$($SQLITE3_BIN <<EOF
.log stderr
.load '$EXTENSION_FILE'
$sql
EOF
    )
    if [ "$result" != "$expected" ]; then
        echo "Failed SQL: $sql"
        echo "Expected:   $expected"
        echo "Actual:     $result"
        exit 1
    fi
}

test_one "SELECT hex(gzip('12345'));"                 "1F8B08000000000000FF333432363105001C3AF5CB05000000"
test_one "SELECT gzip_decode(gzip('12345'));"         "12345"
test_one "SELECT gzip_decode(gzip('12345', 1));"      "12345"
test_one "SELECT gzip_decode(gzip('12345', 9));"      "12345"
test_one "SELECT gzip_test(gzip('12345'));"           "1"

test_one "SELECT hex(brotli('12345'));"               "0B0280313233343503"
test_one "SELECT brotli_decode(brotli('12345'));"     "12345"
test_one "SELECT brotli_decode(brotli('12345', 1));"  "12345"
test_one "SELECT brotli_decode(brotli('12345', 9));"  "12345"
test_one "SELECT brotli_test(brotli('12345'));"       "1"
