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

	User user2 = new User();
	user2.username = "alicecool";
	user2.fullName = "Alice is cool";
	user2.email = "alice@alice.hq";
	db.insert(user2);

	Toot toot = new Toot();
	toot.message = "Hello world!";
	toot.author = user;
	db.insert(toot);

	Comment comment = new Comment();
	comment.replyTo = toot;
	comment.message = "Very cool!";
	comment.author.foreignKey = user2.username;
	db.insert(comment);

	// auto comments = db.select!Comment
	// 	.condition(c => c.author.email.like("alice%"))
	// 	.array;
	// assert(comments.length == 1);
}