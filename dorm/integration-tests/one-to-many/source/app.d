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

void main(string[] args)
{
	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "database.sqlite3"
	};
	auto db = DormDB(options);

	User user = new User();
	user.username = "bobbingnator1";
	user.fullName = "Bob Bobbington";
	user.email = "bob@bobbington.bob";
	db.insert(user);

	Toot toot = new Toot();
	toot.message = "Hello world!";
	toot.author = user;
	// db.insert(toot);
}