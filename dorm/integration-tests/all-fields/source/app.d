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

mixin SetupDormRuntime;

void main()
{
	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "database.sqlite3"
	};
	auto db = DormDB(options);

	@DormPatch!User
	struct UserInsert
	{
		string name = "Bob";
		string password = "123456";
		string email = "bob@example.org";
		ubyte age = 123;
		DateTime birthday = DateTime(1912, 12, 12);
		State state = State.critical;
		string state2 = "unknown";
		bool isAdmin = true;
		string comment = "Very nice person :3";
		long ownPrimaryKey = 123;
		ulong someTimestamp = 1_000_000_000UL;
		int uuid = int.max;
		int someInt = 20;
		Common commonFields = Common("CommonName", SuperCommon(12_345));
	}

	db.insert(UserInsert.init);

	assertThrown({
		// violating unique constraint here
		db.insert(UserInsert.init);
	}());

	size_t total;
	foreach (user; db.select!User.stream)
	{
		total++;

		foreach (i, field; UserInsert.init.tupleof)
			assert(__traits(getMember, user, __traits(identifier, UserInsert.tupleof[i]))
				== field, text("Field ", __traits(identifier, UserInsert.tupleof[i]), " no match: ",
					__traits(getMember, user, __traits(identifier, UserInsert.tupleof[i])),
					" != ", field));
	}

	assert(total == 1);
}