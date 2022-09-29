import models;

import core.thread;
import core.time;
import std.datetime.systime;
import std.range;
import std.stdio;

import dorm.api.db;

mixin SetupDormRuntime;

void main(string[] args)
{
	@DormPatch!User
	struct UserSelection
	{
		string username;
		SysTime createdAt;
	}

	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "database.sqlite3"
	};
	auto db = DormDB(options);

	foreach (i; 0 .. 10)
	{
		auto user = new User();
		user.username = "Jan " ~ cast(char)('A' + i);
		user.password = "123456";
		db.insert(user);
		Thread.sleep(1.seconds);
	}

	auto oldestJans = db.select!UserSelection
		.condition(u => Condition.and(
				u.username.like("Jan%"),
				u.not.isAdmin
			))
		// .order(u => u.createdAt.asc)
		// .take(5)
		.stream();

	writeln("Oldest 5 Jans:");
	foreach (i, jan; oldestJans)
		writefln!"#%d %s\tcreated at %s"(i + 1, jan.username, jan.createdAt);
}