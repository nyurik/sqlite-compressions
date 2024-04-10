#!/usr/bin/env bash
set -euo pipefail

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
test_one "SELECT gzip_test(x'123456');"               "0"

test_one "SELECT hex(brotli('12345'));"               "0B0280313233343503"
test_one "SELECT brotli_decode(brotli('12345'));"     "12345"
test_one "SELECT brotli_decode(brotli('12345', 1));"  "12345"
test_one "SELECT brotli_decode(brotli('12345', 9));"  "12345"
test_one "SELECT brotli_test(brotli('12345'));"       "1"
test_one "SELECT brotli_test(x'123456');"             "0"

test_one "SELECT hex(bzip2('12345'));"               "425A6836314159265359426548B800000008003E0020002183419A025C7177245385090426548B80"
test_one "SELECT bzip2_decode(bzip2('12345'));"     "12345"
test_one "SELECT bzip2_decode(bzip2('12345', 1));"  "12345"
test_one "SELECT bzip2_decode(bzip2('12345', 9));"  "12345"
test_one "SELECT bzip2_test(bzip2('12345'));"       "1"
test_one "SELECT bzip2_test(x'123456');"             "0"

test_one "SELECT hex(bsdiff4('013479', '23456789'));"    "42534449464634302E0000000000000025000000000000000800000000000000425A68363141592653596A17AE4F00000160006E80080020002188C08601CAD80622AF61772453850906A17AE4F0425A6836314159265359B1F7404B00000040004000200021184682EE48A70A12163EE80960425A6836314159265359F715663B00000008001FC02000310C00C4C265CE5DE2EE48A70A121EE2ACC760"
test_one "SELECT bspatch4('013479', bsdiff4('013479', '23456789'));"         "23456789"


echo "------------------------------"
echo "All tests passed successfully!"
