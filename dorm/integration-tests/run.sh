#!/usr/bin/env bash
set -euo pipefail

pushd ../../rorm/
cargo build -p rorm-cli -r
export RORM_CLI="$(pwd)/target/release/rorm-cli"
popd

if [ -z "${1:-}" ];
then
	MATCH="*/"
else
	MATCH="$1"
fi

DATABASE_CONFIG=$(cat <<-END
[Database]
Driver = 'SQLite'
Name = 'database.sqlite3'
Host = '127.0.0.1'
Port = 3306
User = 'user'
Password = 'change_me'
LastMigrationTableName = '_drorm__last_migration'
END
)

EXIT_CODE=0

for testDir in $MATCH
do
	echo "Running $testDir"
	pushd "$testDir"
	if [ -f "run.sh" ]; then
		rm -rf .dub
		rm -rf migrations
		rm -f .models.json
		rm -f database.sqlite3
		echo "$DATABASE_CONFIG" > database.toml
		if ! ./run.sh; then
			echo "Error: Test $testDir failed"
			EXIT_CODE=1
		fi
	else
		echo "Error: Missing run.sh in $testDir"
		EXIT_CODE=1
	fi
	popd
done

exit $EXIT_CODE
