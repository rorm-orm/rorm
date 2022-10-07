import models;

import core.thread;
import core.time;
import std.datetime.systime;
import std.datetime.date;
import std.range;
import std.stdio;

import dorm.api.db;

mixin SetupDormRuntime;

void main(string[] args)
{
	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "database.sqlite3"
	};
	auto db = DormDB(options);

	@DormPatch!User
	struct UserInsert
	{
		SysTime validUntil;
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
		ulong creationTime = 1_000_000_000UL;
		int uuid = int.max;
		int someInt = 20;
		Common commonFields = Common("CommonName", SuperCommon(12345));
	}
	db.insert(UserInsert(Clock.currTime + 8.hours));
}