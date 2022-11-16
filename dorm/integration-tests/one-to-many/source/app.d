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

	User user = new User();
	user.id = "bobbingnator1";
	user.fullName = "Bob Bobbington";
	user.email = "bob@bobbington.bob";
	db.insert(user);

	User user2 = new User();
	user2.id = "alicecool";
	user2.fullName = "Alice is cool";
	user2.email = "alice@alice.hq";
	db.insert(user2);

	Toot toot = new Toot();
	toot.message = "Hello world!";
	toot.author = user;
	db.insert(toot);

	Comment.Fields comment;
	comment.replyTo = toot;
	comment.message = "Very cool!";
	comment.author.foreignKey = user.id;
	db.insert(comment);

	Comment.Fields comment2;
	comment2.replyTo = toot;
	comment2.message = "I like this";
	comment2.author.foreignKey = user2.id;
	db.insert(comment2);

	auto allComments = db.select!Comment
		.array;
	assert(allComments.length == 2);

	auto aliceComments = db.select!Comment
		.condition(c => c.author.email.like("alice%"))
		.array;
	assert(aliceComments.length == 1);
}