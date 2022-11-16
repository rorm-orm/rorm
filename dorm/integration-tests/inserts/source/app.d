import models;

import core.thread;
import core.time;
import std.conv;
import std.datetime.date;
import std.datetime.systime;
import std.exception;
import std.range;
import std.stdio;

import dorm.api.db;
import dorm.declarative.conversion;

mixin SetupDormRuntime;

void main()
{
	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "database.sqlite3"
	};
	auto db = DormDB(options);

	User.Fields userInsert1 = {
		name: "alice_alicington",
		fullName: "Alice Alicington",
		email: "alice@alicingt.on",
		auth: AuthInfo(
			"password",
			"12345678",
			"tok123",
			"secret",
			"???"
		)
	};

	User.Fields userInsert2 = {
		name: "bob_bobbington",
		fullName: "Bob Bobbington",
		email: "bob@bobbingt.on",
		auth: AuthInfo(
			"password",
			"12345678",
			"tok123",
			"secret",
			"???"
		)
	};

	User.Fields userInsert3 = {
		name: "foo_bar",
		fullName: "Foo Bar",
		email: "foo.bar@localhost",
		auth: AuthInfo(
			"password",
			"12345678",
			"tok123",
			"secret",
			"???"
		)
	};

	db.insert(userInsert1);
	assertThrown({
		// violating unique constraint here
		db.insert(userInsert1);
	}());
	db.insert(userInsert2);
	db.insert(userInsert3);

	User.Fields[3] f = [userInsert1, userInsert2, userInsert3];

	size_t total;
	foreach (user; db.select!User.stream)
	{
		assert(user.fields == f[total]);
		total++;
	}

	assert(total == 3);

	db.rawSQL("DELETE FROM " ~ DormLayout!User.tableName)
		.exec();

	foreach (user; db.select!User.stream)
		assert(false, "Deleted from " ~ DormLayout!User.tableName
			~ ", but found row " ~ user.fields.to!string);

	total = 0;
	db.insert(f[]);
	foreach (user; db.select!User.stream)
	{
		assert(user.fields == f[total]);
		total++;
	}

	assert(total == 3);
}