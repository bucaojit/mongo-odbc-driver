#!/bin/bash

# Run this script from top level directory
# ./resources/run_mongosql_adf.sh

RUN_ADF=./resources/run_adf.sh
MONGOSQL_DIR=$1
EXECUTE_GO=local_adf/mongohouse/cmd/mongohoused/internal/execute.go

if [[ -z $1 ]]; then
  MONGOSQL_DIR=mongosql-rs
  if [[ ! -d "./mongosql-rs" ]]; then
    git clone git@github.com:10gen/mongosql-rs.git 
  fi
else
  MONGOSQL_DIR=$1
fi

check_return_code() {
  local return_code=$1
  local error_msg=$2

  if [[ $return_code -ne 0 ]]; then
    echo "ERROR: $error_msg"
    exit 1
  fi
}

# Stop an already running ADF/mongod instance
$RUN_ADF stop

# Run ADF once to download code and setup needed directories
if [[ ! -d "./local_adf/mongohouse/artifacts" ]]; then
  $RUN_ADF start
  check_return_code $? "Failed to start run_adf.sh"
  $RUN_ADF stop
fi

# Update run_adf.sh to not remove library
sed -i '' "s/^    rm -f \$MONGOSQL_LIB/#&/" $RUN_ADF
sed -i '' "s/^    \$GO run cmd\/buildscript\/build.go tools:download:mongosql/#&/" $RUN_ADF

# Build mongosql-rs
( cd $MONGOSQL_DIR && cargo build )

# Copy to artifacts directory
cp $MONGOSQL_DIR/target/debug/libmongosql.a local_adf/mongohouse/artifacts/

# Comment out ADF mongosql version check
# Check if the file already contains the updated line
if ! grep -q 'if err := mongosql.ValidateCLibraryVersion(); err == nil {' $EXECUTE_GO; then
    awk '{
        if ($0 ~ /if err := mongosql.ValidateCLibraryVersion\(\); err != nil {/) {
            print "//" $0
            gsub("err != nil", "err == nil")
            print
        } else {
            print
        }
    }' $EXECUTE_GO > tmpfile && mv tmpfile $EXECUTE_GO
fi


$RUN_ADF start
